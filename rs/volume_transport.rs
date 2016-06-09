extern crate num;
extern crate proust;
use self::num::{Float, FromPrimitive};
use self::proust::*;

use light_volume::*;
use light_field_geom::*;
use optics::*;
use angular_plane::*;
use image_geom::*;
use cl_traits::*;
use spline_kernel::*;

use std::cmp::max;
use std::mem::size_of;

/// Transport for `LightVolume` objects
pub struct VolumeTransport<F: Float> {
    pub geom: LightVolume<F>,
    pub dst: LightFieldGeometry<F>,

    pub overwrite_forw: bool,
    pub overwrite_back: bool,
    pub onto_detector: bool,

    queue: CommandQueue,
    forw_t_kernel: Kernel,
    forw_s_kernel: Kernel,
    back_t_kernel: Kernel,
    back_s_kernel: Kernel,
    scale_kernel: Kernel,
    zero_kernel: Kernel,

    tmp: Mem,                   // half-filtered volume
    scaled: Mem,                // scaled light field
    volume_geom: Mem,           // LightVolumeGeometry
    dst_geom: Mem,              // ImageGeometry
    slice_geom: Mem,            // ImageGeometry

    dst_to_root: Mem,           // Optics
    dst_to_obj: Mem,            // Optics
    forw_spline_kernels_s: Mem, // [SplineKernel]*nz*na
    forw_spline_kernels_t: Mem, // [SplineKernel]*nz*na
    back_spline_kernels_s: Mem, // [SplineKernel]*nz*na
    back_spline_kernels_t: Mem, // [SplineKernel]*nz*na
}

impl<F> VolumeTransport<F>
where F: Float + FromPrimitive {
    pub fn new_simple(src: LightVolume<F>,
                      dst: LightFieldGeometry<F>,
                      to_plane: Optics<F>,
                      queue: CommandQueue) -> Result<Self, Error> {
        Self::new(src, dst, to_plane, true, true, false, queue)
    }

    /// Create a new `VolumeTransport`
    pub fn new(src: LightVolume<F>,
               dst: LightFieldGeometry<F>,
               to_plane: Optics<F>,
               overwrite_forw: bool,
               overwrite_back: bool,
               onto_detector: bool,
               queue: CommandQueue) -> Result<Self, Error> {
        // collect opencl sources
        let sources = match &dst.plane.basis {
            &AngularBasis::Pillbox => {
                [
                    ImageGeometry::<F>::header(),
                    Optics::<F>::header(),
                    LightVolume::<F>::header(),
                    SplineKernel::<F>::header(),
                    include_str!("../cl/transport_pillbox_f32.opencl"),
                    include_str!("../cl/volume_transport_pillbox_f32.opencl"),
                ]
            },
            &AngularBasis::Dirac => {
                [
                    ImageGeometry::<F>::header(),
                    Optics::<F>::header(),
                    LightVolume::<F>::header(),
                    SplineKernel::<F>::header(),
                    include_str!("../cl/transport_dirac_f32.opencl"),
                    include_str!("../cl/volume_transport_dirac_f32.opencl"),
                ]
            },
        };

        // compile opencl code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let unbuilt = try!(Program::new_from_source(context.clone(), &sources));
        let program = try!(unbuilt.build(&[device]));

        // get opencl kernels
        let forw_t_kernel = try!(program.create_kernel("volume_forw_t"));
        let forw_s_kernel = try!(program.create_kernel("volume_forw_s"));
        let back_t_kernel = try!(program.create_kernel("volume_back_t"));
        let back_s_kernel = try!(program.create_kernel("volume_back_s"));
        let scale_kernel = try!(program.create_kernel("volume_scale"));
        let zero_kernel = try!(program.create_kernel("image_zero"));

        // size of temporary buffers
        let tmp_nx = max(src.nx, dst.geom.ns);
        let tmp_ny = max(src.ny, dst.geom.nt);

        // global buffers
        let tmp = try!(queue.create_buffer(size_of::<F>() * tmp_nx * tmp_ny));
        let volume_geom = try!(src.as_cl_buffer(&queue));
        let dst_geom = try!(dst.geom.as_cl_buffer(&queue));
        let slice_geom = try!(src.transaxial_image_geometry().as_cl_buffer(&queue));
        let dst_to_root = try!(dst.to_plane.as_cl_buffer(&queue));
        let dst_to_obj = try!(to_plane.invert().compose(&dst.to_plane).as_cl_buffer(&queue));
        let scaled = try!(queue.create_buffer(size_of::<F>() * dst.geom.ns * dst.geom.nt));

        // slice buffers
        //
        // we precompute the footprints for each angle and slice.  this takes
        // 4 or 6 floats per direction (s and t) and slice (nz) for each angle.
        // in total, this takes
        //      sizeof(T) * (4 | 6) * Na * Nz * 2
        // bytes, which isn't much for reasonable problem sizes
        let mut forw_spline_kernels_s_buf: Vec<u8> = Vec::new();
        let mut forw_spline_kernels_t_buf: Vec<u8> = Vec::new();
        let mut back_spline_kernels_s_buf: Vec<u8> = Vec::new();
        let mut back_spline_kernels_t_buf: Vec<u8> = Vec::new();
        for iz in 0 .. src.nz {
            let slice_lfg = src.slice_light_field_geometry(iz, dst.plane.clone(), to_plane.clone());
            for ia in 0 .. dst.plane.s.len() {
                let (forw_s, forw_t) = slice_lfg.transport_to(&dst, ia);
                let (back_s, back_t) = dst.transport_to(&slice_lfg, ia);

                forw_s.as_cl_bytes(&mut forw_spline_kernels_s_buf);
                forw_t.as_cl_bytes(&mut forw_spline_kernels_t_buf);
                back_s.as_cl_bytes(&mut back_spline_kernels_s_buf);
                back_t.as_cl_bytes(&mut back_spline_kernels_t_buf);
            }
        }

        // load precomputed values onto the GPU
        let forw_spline_kernels_s = try!(queue.create_buffer_from_slice(&forw_spline_kernels_s_buf));
        let forw_spline_kernels_t = try!(queue.create_buffer_from_slice(&forw_spline_kernels_t_buf));
        let back_spline_kernels_s = try!(queue.create_buffer_from_slice(&back_spline_kernels_s_buf));
        let back_spline_kernels_t = try!(queue.create_buffer_from_slice(&back_spline_kernels_t_buf));

        Ok(VolumeTransport{
            geom: src,
            dst: dst,

            overwrite_forw: overwrite_forw,
            overwrite_back: overwrite_back,
            onto_detector: onto_detector,

            queue: queue,
            forw_t_kernel: forw_t_kernel,
            forw_s_kernel: forw_s_kernel,
            back_t_kernel: back_t_kernel,
            back_s_kernel: back_s_kernel,
            scale_kernel: scale_kernel,
            zero_kernel: zero_kernel,

            tmp: tmp,
            volume_geom: volume_geom,
            dst_geom: dst_geom,
            slice_geom: slice_geom,
            scaled: scaled,

            dst_to_root: dst_to_root,
            dst_to_obj: dst_to_obj,
            forw_spline_kernels_s: forw_spline_kernels_s,
            forw_spline_kernels_t: forw_spline_kernels_t,
            back_spline_kernels_s: back_spline_kernels_s,
            back_spline_kernels_t: back_spline_kernels_t,
        })
    }

    fn forw_t(self: &mut Self,
              vol: &Mem,
              ia: usize, iz: usize,
              wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.dst.plane.s.len();
        let u = self.dst.plane.s[ia];
        let v = self.dst.plane.t[ia];
        
        // bind arguments
        try!(self.forw_t_kernel.bind(0, &self.volume_geom));
        try!(self.forw_t_kernel.bind(1, &self.slice_geom));
        try!(self.forw_t_kernel.bind(2, &self.dst_geom));
        try!(self.forw_t_kernel.bind(3, &self.forw_spline_kernels_t));
        try!(self.forw_t_kernel.bind_scalar(4, &(ia as i32)));
        try!(self.forw_t_kernel.bind_scalar(5, &(na as i32)));
        try!(self.forw_t_kernel.bind_scalar(6, &F::to_f32(&u).unwrap()));
        try!(self.forw_t_kernel.bind_scalar(7, &F::to_f32(&v).unwrap()));
        try!(self.forw_t_kernel.bind_scalar(8, &(iz as i32)));
        try!(self.forw_t_kernel.bind(9, vol));
        try!(self.forw_t_kernel.bind_mut(10, &mut self.tmp));

        let local_size = (32, 8, 1);
        let global_size = (self.geom.nx, self.dst.geom.nt, 1);

        self.queue.run_with_events(&mut self.forw_t_kernel,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    fn forw_s(self: &mut Self,
              dst: &mut Mem,
              ia: usize, iz: usize,
              wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.dst.plane.s.len();
        let u = self.dst.plane.s[ia];
        let v = self.dst.plane.t[ia];
        let scale = if self.onto_detector {
            self.geom.dz
        } else {
            self.geom.dz / self.dst.pixel_volume()
        };

        // bind arguments
        try!(self.forw_s_kernel.bind(0, &self.volume_geom));
        try!(self.forw_s_kernel.bind(1, &self.slice_geom));
        try!(self.forw_s_kernel.bind(2, &self.dst_geom));
        try!(self.forw_s_kernel.bind(3, &self.forw_spline_kernels_s));
        try!(self.forw_s_kernel.bind_scalar(4, &(ia as i32)));
        try!(self.forw_s_kernel.bind_scalar(5, &(na as i32)));
        try!(self.forw_s_kernel.bind_scalar(6, &F::to_f32(&u).unwrap()));
        try!(self.forw_s_kernel.bind_scalar(7, &F::to_f32(&v).unwrap()));
        try!(self.forw_s_kernel.bind_scalar(8, &(iz as i32)));
        try!(self.forw_s_kernel.bind_scalar(9, &scale));
        try!(self.forw_s_kernel.bind(10, &self.tmp));
        try!(self.forw_s_kernel.bind_mut(11, dst));

        let local_size = (32, 8, 1);
        let global_size = (self.dst.geom.nt, self.dst.geom.ns, 1);

        self.queue.run_with_events(&mut self.forw_s_kernel,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    fn back_t(self: &mut Self,
              dst: &Mem,
              ia: usize, iz: usize,
              wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.dst.plane.s.len();
        let u = self.dst.plane.s[ia];
        let v = self.dst.plane.t[ia];
        let scale = if self.onto_detector {
            self.geom.dz
        } else {
            self.geom.dz / self.dst.pixel_volume()
        };

        // bind arguments
        try!(self.back_t_kernel.bind(0, &self.volume_geom));
        try!(self.back_t_kernel.bind(1, &self.slice_geom));
        try!(self.back_t_kernel.bind(2, &self.dst_geom));
        try!(self.back_t_kernel.bind(3, &self.back_spline_kernels_t));
        try!(self.back_t_kernel.bind_scalar(4, &(ia as i32)));
        try!(self.back_t_kernel.bind_scalar(5, &(na as i32)));
        try!(self.back_t_kernel.bind_scalar(6, &F::to_f32(&u).unwrap()));
        try!(self.back_t_kernel.bind_scalar(7, &F::to_f32(&v).unwrap()));
        try!(self.back_t_kernel.bind_scalar(8, &(iz as i32)));
        try!(self.back_t_kernel.bind_scalar(9, &scale));
        try!(self.back_t_kernel.bind(10, dst));
        try!(self.back_t_kernel.bind_mut(11, &mut self.tmp));

        let local_size = (32, 8, 1);
        let global_size = (self.dst.geom.ns, self.geom.ny, 1);

        self.queue.run_with_events(&mut self.back_t_kernel,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    fn back_s(self: &mut Self,
              vol: &mut Mem,
              ia: usize, iz: usize,
              wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.dst.plane.s.len();
        let u = self.dst.plane.s[ia];
        let v = self.dst.plane.t[ia];
        let overwrite_flag = if self.overwrite_back {
            1u32
        } else {
            0u32
        };

        // bind arguments
        try!(self.back_s_kernel.bind(0, &self.volume_geom));
        try!(self.back_s_kernel.bind(1, &self.slice_geom));
        try!(self.back_s_kernel.bind(2, &self.dst_geom));
        try!(self.back_s_kernel.bind(3, &self.back_spline_kernels_s));
        try!(self.back_s_kernel.bind_scalar(4, &(ia as i32)));
        try!(self.back_s_kernel.bind_scalar(5, &(na as i32)));
        try!(self.back_s_kernel.bind_scalar(6, &F::to_f32(&u).unwrap()));
        try!(self.back_s_kernel.bind_scalar(7, &F::to_f32(&v).unwrap()));
        try!(self.back_s_kernel.bind_scalar(8, &(iz as i32)));
        try!(self.back_s_kernel.bind(9, &self.tmp));
        try!(self.back_s_kernel.bind_mut(10, vol));
        try!(self.back_s_kernel.bind_scalar(11, &overwrite_flag));

        let local_size = (32, 8, 1);
        let global_size = (self.geom.ny, self.geom.nx, 1);

        self.queue.run_with_events(&mut self.back_s_kernel,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    fn scale(self: &mut Self,
             input: &Mem,
             output: &mut Mem,
             ia: usize,
             wait_for: &[Event],
             overwrite: bool) -> Result<Event, Error> {
        let s = self.dst.plane.s[ia];
        let t = self.dst.plane.t[ia];
        let overwrite_flag = if overwrite {
            1u32
        } else {
            0u32
        };

        // bind arguments
        try!(self.scale_kernel.bind(0, &self.dst_geom));
        try!(self.scale_kernel.bind(1, &self.dst_to_root));
        try!(self.scale_kernel.bind(2, &self.dst_to_obj));
        try!(self.scale_kernel.bind_scalar(3, &F::to_f32(&s).unwrap()));
        try!(self.scale_kernel.bind_scalar(4, &F::to_f32(&t).unwrap()));
        try!(self.scale_kernel.bind(5, input));
        try!(self.scale_kernel.bind_mut(6, output));
        try!(self.scale_kernel.bind_scalar(7, &overwrite_flag));

        let local_size = (32, 8, 1);
        let global_size = (self.dst.geom.ns, self.dst.geom.nt, 1);

        self.queue.run_with_events(&mut self.scale_kernel,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    fn zero(self: &mut Self,
            img: &mut Mem,
            wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.zero_kernel.bind(0, &self.dst_geom));
        try!(self.zero_kernel.bind_mut(1, img));

        let local_size = (32, 8, 1);
        let global_size = (self.dst.geom.ns, self.dst.geom.nt, 1);

        self.queue.run_with_events(&mut self.zero_kernel,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    pub fn forw(self: &mut Self,
                vol: &Mem,
                dst: &mut Mem,
                ia: usize,
                wait_for: &[Event]) -> Result<Event, Error> {
        let mut tmp_buf = self.scaled.clone();
        let mut evt = try!(self.zero(&mut tmp_buf, wait_for));

        evt = try!(self.forw_t(vol, ia, 0, &[evt]));
        evt = try!(self.forw_s(&mut tmp_buf, ia, 0, &[evt]));

        for iz in 1 .. self.geom.nz {
            evt = try!(self.forw_t(vol, ia, iz, &[evt]));
            evt = try!(self.forw_s(&mut tmp_buf, ia, iz, &[evt]));
        }

        let overwrite_forw = self.overwrite_forw;
        self.scale(&tmp_buf, dst, ia, &[evt], overwrite_forw)
    }

    pub fn back(self: &mut Self,
                dst: &Mem,
                vol: &mut Mem,
                ia: usize,
                wait_for: &[Event]) -> Result<Event, Error> {
        let mut scaled_copy = self.scaled.clone();
        let mut evt = try!(self.scale(dst, &mut scaled_copy, ia, wait_for, true));

        for iz in 0 .. self.geom.nz {
            evt = try!(self.back_t(&scaled_copy, ia, iz, &[evt]));
            evt = try!(self.back_s(vol, ia, iz, &[evt]));
        }

        Ok(evt)
    }
}

#[test]
fn test_volume_dirac() {
    use env::*;
    use lens::*;
    use geom::*;

    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

   let lens = Lens{
        center_s: 1f32,
        center_t: -1.5f32,
        radius_s: 20f32,
        radius_t: 15f32,
        focal_length_s: 30f32,
        focal_length_t: 35f32,
    };
    let plane = lens.as_angular_plane(AngularBasis::Dirac, 20);

    let vg = LightVolume{
        nx: 100,
        ny: 200,
        nz: 100,
        dx: 1.0,
        dy: 1.1,
        dz: 1.0,
        offset_x: 0.5,
        offset_y: 2.9,
        offset_z: 0.0,
        opaque: false,
    };

    let dst_geom = ImageGeometry{
        ns: 512,
        nt: 768,
        ds: 5e-2,
        dt: 3e-2,
        offset_s: -4.0,
        offset_t: 2.1,
    };

    let dst = LightFieldGeometry{
        geom: dst_geom.clone(),
        plane: plane.clone(),
        to_plane: Optics::translation(&40f32),
    };

    let to_plane = lens.optics().then(&Optics::translation(&500f32)).invert();

    let mut xport = VolumeTransport::new_simple(vg.clone(), 
                                                dst, to_plane, queue.clone()).unwrap();

    let x = vg.rands();
    let y = dst_geom.rands();
    let mut cx = dst_geom.zeros();
    let mut cty = vg.zeros();

    let x_buf = queue.create_buffer_from_slice(&x).unwrap();
    let y_buf = queue.create_buffer_from_slice(&y).unwrap();
    let mut cx_buf = dst_geom.zeros_buf(&queue).unwrap();
    let mut cty_buf = vg.zeros_buf(&queue).unwrap();

    xport.forw(&x_buf, &mut cx_buf, 50, &[]).unwrap().wait().unwrap();
    xport.back(&y_buf, &mut cty_buf, 50, &[]).unwrap().wait().unwrap();
    queue.read_buffer(&cx_buf, &mut cx).unwrap();
    queue.read_buffer(&cty_buf, &mut cty).unwrap();

    let v1 = cx.iter().zip(y.iter()).fold(0f32, |s, (ui, vi)| s + ui*vi);
    let v2 = cty.iter().zip(x.iter()).fold(0f32, |s, (ui, vi)| s + ui*vi);
    let nrmse = (v1 - v2).abs() / v1.abs().max(v2.abs());

    println!("Adjoint NRMSE for VolumeTransport-Dirac: {}", nrmse);
    println!("y'Cx = {}", v1);
    println!("(C'y)'x = {}", v2);

    assert!(nrmse < 1e-2);
}

#[test]
fn test_volume_pillbox() {
    use env::*;
    use lens::*;
    use geom::*;

    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

   let lens = Lens{
        center_s: 1f32,
        center_t: -1.5f32,
        radius_s: 20f32,
        radius_t: 15f32,
        focal_length_s: 30f32,
        focal_length_t: 35f32,
    };
    let plane = lens.as_angular_plane(AngularBasis::Pillbox, 20);

    let vg = LightVolume{
        nx: 100,
        ny: 200,
        nz: 100,
        dx: 1.0,
        dy: 1.1,
        dz: 1.0,
        offset_x: 0.5,
        offset_y: 2.9,
        offset_z: 0.0,
        opaque: false,
    };

    let dst_geom = ImageGeometry{
        ns: 512,
        nt: 768,
        ds: 5e-2,
        dt: 3e-2,
        offset_s: -4.0,
        offset_t: 2.1,
    };

    let dst = LightFieldGeometry{
        geom: dst_geom.clone(),
        plane: plane.clone(),
        to_plane: Optics::translation(&40f32),
    };

    let to_plane = lens.optics().then(&Optics::translation(&500f32)).invert();

    let mut xport = VolumeTransport::new_simple(vg.clone(), 
                                                dst, to_plane, queue.clone()).unwrap();

    let x = vg.rands();
    let y = dst_geom.rands();
    let mut cx = dst_geom.zeros();
    let mut cty = vg.zeros();

    let x_buf = queue.create_buffer_from_slice(&x).unwrap();
    let y_buf = queue.create_buffer_from_slice(&y).unwrap();
    let mut cx_buf = dst_geom.rands_buf(&queue).unwrap();
    let mut cty_buf = vg.rands_buf(&queue).unwrap();

    xport.forw(&x_buf, &mut cx_buf, 50, &[]).unwrap().wait().unwrap();
    xport.back(&y_buf, &mut cty_buf, 50, &[]).unwrap().wait().unwrap();
    queue.read_buffer(&cx_buf, &mut cx).unwrap();
    queue.read_buffer(&cty_buf, &mut cty).unwrap();

    let v1 = cx.iter().zip(y.iter()).fold(0f32, |s, (ui, vi)| s + ui*vi);
    let v2 = cty.iter().zip(x.iter()).fold(0f32, |s, (ui, vi)| s + ui*vi);
    let nrmse = (v1 - v2).abs() / v1.abs().max(v2.abs());

    println!("Adjoint NRMSE for VolumeTransport-Pillbox: {}", nrmse);
    println!("y'Cx = {}", v1);
    println!("(C'y)'x = {}", v2);

    assert!(nrmse < 1e-2);
}

