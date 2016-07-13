extern crate num;
extern crate nalgebra;
extern crate proust;
use isometry::*;
use self::nalgebra::{Rotation3, BaseFloat, ApproxEq};
use self::num::{Float, FromPrimitive};
use light_volume::*;
use self::proust::*;
use cl_traits::*;
use geom::*;
use optics::*;
use image_geom::*;
use spline_kernel::*;

/// Rotates a LightVolume
pub struct VolumeRotation<F: Float> {
    pub src_geom: LightVolume<F>,
    pub dst_geom: LightVolume<F>,

    queue: CommandQueue,
    filter_x: Kernel,
    filter_y: Kernel,
    filter_z: Kernel,
    tmp: Mem,

    spline_forw_x: Mem,
    spline_forw_y: Mem,
    spline_forw_z: Mem,

    spline_back_x: Mem,
    spline_back_y: Mem,
    spline_back_z: Mem,

    dst_geom_buf: Mem,
}

fn fmin<F: Float>(x: F, y: F) -> F {
    if x < y {
        x
    } else {
        y
    }
}

fn fmax<F: Float>(x: F, y: F) -> F {
    if x > y {
        x
    } else {
        y
    }
}

fn trap_weight<F: Float>(mut step0: F, mut step1: F, mut ds: F) -> F {
    let c2 = F::one() + F::one();
    step0 = (step0 / c2).abs();
    step1 = (step1 / c2).abs();
    ds = (ds / c2).abs();

    let min_step = fmin(step0, step1);
    let max_step = fmax(step0, step1);

    let tau1 = fmin(ds, max_step - min_step);
    let tau2 = fmin(fmax(ds, max_step - min_step), max_step + min_step);

    let tw = tau2 - (max_step - min_step) -
             (tau2 - (max_step - min_step)).powi(2) / (c2 * c2 * min_step);

    c2 * (tw + tau1)
}

impl<F: Float> ClHeader for VolumeRotation<F> {
    fn header() -> &'static str {
        include_str!("../cl/volume_rotation_f32.opencl")
    }
}

impl<F: Float + BaseFloat + ApproxEq<F> + FromPrimitive> VolumeRotation<F> {
    pub fn new(rotation: &Rotation3<F>,
               src_geom: LightVolume<F>,
               queue: CommandQueue)
               -> Result<Self, Error> {
        // get OpenCL objects and source code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let sources = &[Optics::<F>::header(),
                        ImageGeometry::<F>::header(),
                        LightVolume::<F>::header(),
                        SplineKernel::<F>::header(),
                        Self::header()];

        // compile program
        let unbuilt = try!(Program::new_from_source(context.clone(), sources));
        let program = try!(unbuilt.build(&[device]));

        // get opencl kernels
        let filter_x = try!(program.create_kernel("rotate_filter_x"));
        let filter_y = try!(program.create_kernel("rotate_filter_y"));
        let filter_z = try!(program.create_kernel("rotate_filter_z"));

        let shear_decomp = ShearDecomposition::new(rotation);
        let dst_geom = src_geom.scale(shear_decomp.sx,
                                      shear_decomp.sy,
                                      shear_decomp.sz);

        let dst_geom_buf = try!(dst_geom.as_cl_buffer(&queue));
        let tmp = try!(src_geom.zeros_buf(&queue));

        // compute spline footprints
        let mut forw_x_buf: Vec<u8> = Vec::new();
        let mut forw_y_buf: Vec<u8> = Vec::new();
        let mut forw_z_buf: Vec<u8> = Vec::new();

        let mut back_x_buf: Vec<u8> = Vec::new();
        let mut back_y_buf: Vec<u8> = Vec::new();
        let mut back_z_buf: Vec<u8> = Vec::new();

        let c2 = F::one() + F::one();

        // z splines
        // TRICK: we use "1" for the height of the rotation kernels because
        // multi-camera calibration is a wash anyway and it removes a
        // degeracy at >= 40 degree rotations.
        let hx_z = fmin(dst_geom.dx.abs(), (dst_geom.dz / shear_decomp.zx).abs());
        let hy_z = trap_weight(dst_geom.dz / shear_decomp.zy,
                               dst_geom.dx * shear_decomp.zx / shear_decomp.zy,
                               dst_geom.dy);
        let hz = hx_z * hy_z / dst_geom.voxel_volume();
        for iy in 0..dst_geom.ny {
            let y = dst_geom.iy2y(iy);
            for ix in 0..dst_geom.nx {
                let x = dst_geom.ix2x(ix);

                let taus_forw = vec![
                    dst_geom.dz/c2 - shear_decomp.zx*(x + dst_geom.dx/c2) - shear_decomp.zy*(y + dst_geom.dy/c2),
                    dst_geom.dz/c2 - shear_decomp.zx*(x + dst_geom.dx/c2) - shear_decomp.zy*(y - dst_geom.dy/c2),
                    dst_geom.dz/c2 - shear_decomp.zx*(x - dst_geom.dx/c2) - shear_decomp.zy*(y + dst_geom.dy/c2),
                    dst_geom.dz/c2 - shear_decomp.zx*(x - dst_geom.dx/c2) - shear_decomp.zy*(y - dst_geom.dy/c2),
                    -dst_geom.dz/c2 - shear_decomp.zx*(x + dst_geom.dx/c2) - shear_decomp.zy*(y + dst_geom.dy/c2),
                    -dst_geom.dz/c2 - shear_decomp.zx*(x + dst_geom.dx/c2) - shear_decomp.zy*(y - dst_geom.dy/c2),
                    -dst_geom.dz/c2 - shear_decomp.zx*(x - dst_geom.dx/c2) - shear_decomp.zy*(y + dst_geom.dy/c2),
                    -dst_geom.dz/c2 - shear_decomp.zx*(x - dst_geom.dx/c2) - shear_decomp.zy*(y - dst_geom.dy/c2),
                ];
                let forw = SplineKernel::new_quad(hz, F::one(), &taus_forw);
                forw.as_cl_bytes(&mut forw_z_buf);

                let taus_back = vec![
                    dst_geom.dz/c2 + shear_decomp.zx*(x + dst_geom.dx/c2) + shear_decomp.zy*(y + dst_geom.dy/c2),
                    dst_geom.dz/c2 + shear_decomp.zx*(x + dst_geom.dx/c2) + shear_decomp.zy*(y - dst_geom.dy/c2),
                    dst_geom.dz/c2 + shear_decomp.zx*(x - dst_geom.dx/c2) + shear_decomp.zy*(y + dst_geom.dy/c2),
                    dst_geom.dz/c2 + shear_decomp.zx*(x - dst_geom.dx/c2) + shear_decomp.zy*(y - dst_geom.dy/c2),
                    -dst_geom.dz/c2 + shear_decomp.zx*(x + dst_geom.dx/c2) + shear_decomp.zy*(y + dst_geom.dy/c2),
                    -dst_geom.dz/c2 + shear_decomp.zx*(x + dst_geom.dx/c2) + shear_decomp.zy*(y - dst_geom.dy/c2),
                    -dst_geom.dz/c2 + shear_decomp.zx*(x - dst_geom.dx/c2) + shear_decomp.zy*(y + dst_geom.dy/c2),
                    -dst_geom.dz/c2 + shear_decomp.zx*(x - dst_geom.dx/c2) + shear_decomp.zy*(y - dst_geom.dy/c2),
                ];
                let back = SplineKernel::new_quad(hz, F::one(), &taus_back);
                back.as_cl_bytes(&mut back_z_buf);
            }
        }

        // y splines
        let hx_y = fmin(dst_geom.dx.abs(), (dst_geom.dy / shear_decomp.yx).abs());
        let hz_y = trap_weight(dst_geom.dy / shear_decomp.yz,
                               dst_geom.dx * shear_decomp.yx / shear_decomp.yz,
                               dst_geom.dz);
        let hy = hx_y * hz_y / dst_geom.voxel_volume();
        for iz in 0..dst_geom.nz {
            let z = dst_geom.iz2z(iz);
            for ix in 0..dst_geom.nx {
                let x = dst_geom.ix2x(ix);

                let taus_forw = vec![
                    dst_geom.dy/c2 -shear_decomp.yx*(x + dst_geom.dx/c2) - shear_decomp.yz*(z + dst_geom.dz/c2),
                    dst_geom.dy/c2 -shear_decomp.yx*(x + dst_geom.dx/c2) - shear_decomp.yz*(z - dst_geom.dz/c2),
                    dst_geom.dy/c2 -shear_decomp.yx*(x - dst_geom.dx/c2) - shear_decomp.yz*(z + dst_geom.dz/c2),
                    dst_geom.dy/c2 -shear_decomp.yx*(x - dst_geom.dx/c2) - shear_decomp.yz*(z - dst_geom.dz/c2),
                    -dst_geom.dy/c2 -shear_decomp.yx*(x + dst_geom.dx/c2) - shear_decomp.yz*(z + dst_geom.dz/c2),
                    -dst_geom.dy/c2 -shear_decomp.yx*(x + dst_geom.dx/c2) - shear_decomp.yz*(z - dst_geom.dz/c2),
                    -dst_geom.dy/c2 -shear_decomp.yx*(x - dst_geom.dx/c2) - shear_decomp.yz*(z + dst_geom.dz/c2),
                    -dst_geom.dy/c2 -shear_decomp.yx*(x - dst_geom.dx/c2) - shear_decomp.yz*(z - dst_geom.dz/c2),
                ];
                let forw = SplineKernel::new_quad(hy, F::one(), &taus_forw);
                forw.as_cl_bytes(&mut forw_y_buf);

                let taus_back = vec![
                    dst_geom.dy/c2 +shear_decomp.yx*(x + dst_geom.dx/c2) + shear_decomp.yz*(z + dst_geom.dz/c2),
                    dst_geom.dy/c2 +shear_decomp.yx*(x + dst_geom.dx/c2) + shear_decomp.yz*(z - dst_geom.dz/c2),
                    dst_geom.dy/c2 +shear_decomp.yx*(x - dst_geom.dx/c2) + shear_decomp.yz*(z + dst_geom.dz/c2),
                    dst_geom.dy/c2 +shear_decomp.yx*(x - dst_geom.dx/c2) + shear_decomp.yz*(z - dst_geom.dz/c2),
                    -dst_geom.dy/c2 +shear_decomp.yx*(x + dst_geom.dx/c2) + shear_decomp.yz*(z + dst_geom.dz/c2),
                    -dst_geom.dy/c2 +shear_decomp.yx*(x + dst_geom.dx/c2) + shear_decomp.yz*(z - dst_geom.dz/c2),
                    -dst_geom.dy/c2 +shear_decomp.yx*(x - dst_geom.dx/c2) + shear_decomp.yz*(z + dst_geom.dz/c2),
                    -dst_geom.dy/c2 +shear_decomp.yx*(x - dst_geom.dx/c2) + shear_decomp.yz*(z - dst_geom.dz/c2),
                ];
                let back = SplineKernel::new_quad(hy, F::one(), &taus_back);
                back.as_cl_bytes(&mut back_y_buf);
            }
        }

        // forw x
        let hy_x = fmin(dst_geom.dy.abs(), (dst_geom.dx / shear_decomp.xy).abs());
        let hz_x = trap_weight(dst_geom.dx / shear_decomp.xz,
                               dst_geom.dy * shear_decomp.xy / shear_decomp.xz,
                               dst_geom.dx);
        let hx = hz_x * hy_x / dst_geom.voxel_volume();
        for iz in 0..dst_geom.nz {
            let z = dst_geom.iz2z(iz);
            for iy in 0..dst_geom.ny {
                let y = dst_geom.iy2y(iy);

                let taus_forw = vec![
                    dst_geom.dx/c2 -shear_decomp.xy*(y + dst_geom.dy/c2) - shear_decomp.xz*(z + dst_geom.dz/c2),
                    dst_geom.dx/c2 -shear_decomp.xy*(y + dst_geom.dy/c2) - shear_decomp.xz*(z - dst_geom.dz/c2),
                    dst_geom.dx/c2 -shear_decomp.xy*(y - dst_geom.dy/c2) - shear_decomp.xz*(z + dst_geom.dz/c2),
                    dst_geom.dx/c2 -shear_decomp.xy*(y - dst_geom.dy/c2) - shear_decomp.xz*(z - dst_geom.dz/c2),
                    -dst_geom.dx/c2 -shear_decomp.xy*(y + dst_geom.dy/c2) - shear_decomp.xz*(z + dst_geom.dz/c2),
                    -dst_geom.dx/c2 -shear_decomp.xy*(y + dst_geom.dy/c2) - shear_decomp.xz*(z - dst_geom.dz/c2),
                    -dst_geom.dx/c2 -shear_decomp.xy*(y - dst_geom.dy/c2) - shear_decomp.xz*(z + dst_geom.dz/c2),
                    -dst_geom.dx/c2 -shear_decomp.xy*(y - dst_geom.dy/c2) - shear_decomp.xz*(z - dst_geom.dz/c2),
                ];
                let forw = SplineKernel::new_quad(hx, F::one(), &taus_forw);
                forw.as_cl_bytes(&mut forw_x_buf);

                let taus_back = vec![
                    dst_geom.dx/c2 +shear_decomp.xy*(y + dst_geom.dy/c2) + shear_decomp.xz*(z + dst_geom.dz/c2),
                    dst_geom.dx/c2 +shear_decomp.xy*(y + dst_geom.dy/c2) + shear_decomp.xz*(z - dst_geom.dz/c2),
                    dst_geom.dx/c2 +shear_decomp.xy*(y - dst_geom.dy/c2) + shear_decomp.xz*(z + dst_geom.dz/c2),
                    dst_geom.dx/c2 +shear_decomp.xy*(y - dst_geom.dy/c2) + shear_decomp.xz*(z - dst_geom.dz/c2),
                    -dst_geom.dx/c2 +shear_decomp.xy*(y + dst_geom.dy/c2) + shear_decomp.xz*(z + dst_geom.dz/c2),
                    -dst_geom.dx/c2 +shear_decomp.xy*(y + dst_geom.dy/c2) + shear_decomp.xz*(z - dst_geom.dz/c2),
                    -dst_geom.dx/c2 +shear_decomp.xy*(y - dst_geom.dy/c2) + shear_decomp.xz*(z + dst_geom.dz/c2),
                    -dst_geom.dx/c2 +shear_decomp.xy*(y - dst_geom.dy/c2) + shear_decomp.xz*(z - dst_geom.dz/c2),
                ];
                let back = SplineKernel::new_quad(hx, F::one(), &taus_back);
                back.as_cl_bytes(&mut back_x_buf);
            }
        }

        let forw_x = try!(queue.create_buffer_from_slice(&forw_x_buf));
        let forw_y = try!(queue.create_buffer_from_slice(&forw_y_buf));
        let forw_z = try!(queue.create_buffer_from_slice(&forw_z_buf));

        let back_x = try!(queue.create_buffer_from_slice(&back_x_buf));
        let back_y = try!(queue.create_buffer_from_slice(&back_y_buf));
        let back_z = try!(queue.create_buffer_from_slice(&back_z_buf));

        Ok(VolumeRotation {
            src_geom: src_geom,
            dst_geom: dst_geom,

            queue: queue,
            filter_x: filter_x,
            filter_y: filter_y,
            filter_z: filter_z,
            tmp: tmp,

            spline_forw_x: forw_x,
            spline_forw_y: forw_y,
            spline_forw_z: forw_z,

            spline_back_x: back_x,
            spline_back_y: back_y,
            spline_back_z: back_z,

            dst_geom_buf: dst_geom_buf,
        })
    }

    fn forw_z(self: &mut Self,
              vol: &Mem,
              out: &mut Mem,
              wait_for: &[Event])
              -> Result<Event, Error> {
        try!(self.filter_z.bind(0, &self.dst_geom_buf));
        try!(self.filter_z.bind(1, &self.spline_forw_z));
        try!(self.filter_z.bind(2, vol));
        try!(self.filter_z.bind(3, out));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_z, local_size, global_size, wait_for)
    }

    fn forw_x(self: &mut Self, out: &Mem, wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.filter_x.bind(0, &self.dst_geom_buf));
        try!(self.filter_x.bind(1, &self.spline_forw_x));
        try!(self.filter_x.bind(2, &out));
        try!(self.filter_x.bind(3, &mut self.tmp));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_x, local_size, global_size, wait_for)
    }

    fn forw_y(self: &mut Self, out: &mut Mem, wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.filter_y.bind(0, &self.dst_geom_buf));
        try!(self.filter_y.bind(1, &self.spline_forw_y));
        try!(self.filter_y.bind(2, &self.tmp));
        try!(self.filter_y.bind(3, out));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_y, local_size, global_size, wait_for)
    }

    pub fn forw(self: &mut Self,
                vol: &Mem,
                out: &mut Mem,
                wait_for: &[Event])
                -> Result<Event, Error> {
        let mut evt = try!(self.forw_z(vol, out, wait_for));
        evt = try!(self.forw_x(out, &[evt]));
        self.forw_y(out, &[evt])
    }

    fn back_y(self: &mut Self,
              vol: &Mem,
              out: &mut Mem,
              wait_for: &[Event])
              -> Result<Event, Error> {
        try!(self.filter_y.bind(0, &self.dst_geom_buf));
        try!(self.filter_y.bind(1, &self.spline_back_y));
        try!(self.filter_y.bind(2, vol));
        try!(self.filter_y.bind(3, out));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_y, local_size, global_size, wait_for)
    }

    fn back_x(self: &mut Self, out: &Mem, wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.filter_x.bind(0, &self.dst_geom_buf));
        try!(self.filter_x.bind(1, &self.spline_back_x));
        try!(self.filter_x.bind(2, &out));
        try!(self.filter_x.bind(3, &mut self.tmp));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_x, local_size, global_size, wait_for)
    }

    fn back_z(self: &mut Self, out: &mut Mem, wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.filter_z.bind(0, &self.dst_geom_buf));
        try!(self.filter_z.bind(1, &self.spline_back_z));
        try!(self.filter_z.bind(2, &self.tmp));
        try!(self.filter_z.bind(3, out));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_z, local_size, global_size, wait_for)
    }

    pub fn back(self: &mut Self,
                vol: &Mem,
                out: &mut Mem,
                wait_for: &[Event])
                -> Result<Event, Error> {
        let mut evt = try!(self.back_y(vol, out, wait_for));
        evt = try!(self.back_x(out, &[evt]));
        self.back_z(out, &[evt])
    }
}

#[test]
fn test_volume_rotation() {
    use env::*;

    let rot: Rotation<f32> = Rotation::new_with_euler_angles(0.2, 0.4, 0.1);
    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

    let src_geom = LightVolume {
        nx: 100,
        ny: 100,
        nz: 100,
        dx: 1.0,
        dy: 1.0,
        dz: 1.0,
        offset_x: 0.0,
        offset_y: 0.0,
        offset_z: 0.0,
        opaque: false,
    };

    let mut rotator = VolumeRotation::new(&rot, src_geom.clone(), queue.clone()).unwrap();

    let u_vec = src_geom.rands();
    let v_vec = src_geom.rands();

    let u = queue.create_buffer_from_slice(&u_vec).unwrap();
    let v = queue.create_buffer_from_slice(&v_vec).unwrap();

    let mut rot_u = src_geom.zeros_buf(&queue).unwrap();
    let mut v_rot = src_geom.zeros_buf(&queue).unwrap();

    rotator.forw(&u, &mut rot_u, &[]).unwrap().wait().unwrap();
    rotator.back(&v, &mut v_rot, &[]).unwrap().wait().unwrap();

    let mut rot_u_vec = src_geom.zeros();
    let mut v_rot_vec = src_geom.zeros();

    queue.read_buffer(&rot_u, &mut rot_u_vec).unwrap().wait().unwrap();
    queue.read_buffer(&v_rot, &mut v_rot_vec).unwrap().wait().unwrap();

    let v1 = rot_u_vec.iter().zip(v_vec.iter()).fold(0f32, |s, (ui, vi)| s + ui * vi);
    let v2 = v_rot_vec.iter().zip(u_vec.iter()).fold(0f32, |s, (vi, ui)| s + ui * vi);
    let nrmse = (v1 - v2).abs() / v1.abs().max(v2.abs());

    println!("Adjoint NRMSE for Volume Rotation: {}", nrmse);
    println!("v'Tu: {}", v1);
    println!("(T'v)'u: {}", v2);
    assert!(nrmse < 1e-4);
}
