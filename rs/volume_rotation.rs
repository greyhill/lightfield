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

    dst_geom_buf: Mem,
}

fn fmin<F: Float>(x: F, y: F) -> F {
    if x < y {
        x
    } else {
        y
    }
}

impl<F: Float> ClHeader for VolumeRotation<F> {
    fn header() -> &'static str {
        include_str!("../cl/volume_rotation_f32.opencl")
    }
}

impl<F: Float + BaseFloat + ApproxEq<F> + FromPrimitive> VolumeRotation<F> {
    pub fn new(rotation: &Rotation3<F>,
               src_geom: LightVolume<F>,
               queue: CommandQueue) -> Result<Self, Error> {
        // get OpenCL objects and source code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let sources = &[
            Optics::<F>::header(),
            ImageGeometry::<F>::header(),
            LightVolume::<F>::header(),
            SplineKernel::<F>::header(),
            Self::header(),
        ];

        // compile program
        let unbuilt = try!(Program::new_from_source(context.clone(), sources));
        let program = try!(unbuilt.build(&[device]));

        // get opencl kernels
        let filter_x = try!(program.create_kernel("rotate_filter_x"));
        let filter_y = try!(program.create_kernel("rotate_filter_y"));
        let filter_z = try!(program.create_kernel("rotate_filter_z"));

        let shear_decomp = ShearDecomposition::new(rotation);
        let dst_geom = src_geom.scale(F::one() / shear_decomp.sx,
                                      F::one() / shear_decomp.sy,
                                      F::one() / shear_decomp.sz);

        let dst_geom_buf = try!(dst_geom.as_cl_buffer(&queue));
        let tmp = try!(src_geom.zeros_buf(&queue));

        // compute spline footprints
        let mut forw_x_buf: Vec<u8> = Vec::new();
        let mut forw_y_buf: Vec<u8> = Vec::new();
        let mut forw_z_buf: Vec<u8> = Vec::new();
        let c2 = F::one() + F::one();

        // forw z
        let hx_forw_z = (dst_geom.dz / shear_decomp.zx).abs();
        let hy_forw_z = fmin((dst_geom.dx*shear_decomp.zx/shear_decomp.zy).abs(), dst_geom.dy.abs());
        for iy in 0 .. dst_geom.ny {
            let y = dst_geom.iy2y(iy);
            for ix in 0 .. dst_geom.nx {
                let x = dst_geom.ix2x(ix);

                let taus_forw = vec![
                    -shear_decomp.zx*(x + dst_geom.dx/c2) - shear_decomp.zy*(y + dst_geom.dy/c2),
                    -shear_decomp.zx*(x + dst_geom.dx/c2) - shear_decomp.zy*(y - dst_geom.dy/c2),
                    -shear_decomp.zx*(x - dst_geom.dx/c2) - shear_decomp.zy*(y + dst_geom.dy/c2),
                    -shear_decomp.zx*(x - dst_geom.dx/c2) - shear_decomp.zy*(y - dst_geom.dy/c2),
                ];
                let forw = SplineKernel::new_trapezoid(hx_forw_z * hy_forw_z, F::one(), &taus_forw);
                forw.as_cl_bytes(&mut forw_z_buf);
            }
        }

        // forw y
        let hx_forw_y = (dst_geom.dy / shear_decomp.yx).abs();
        let hz_forw_y = fmin((dst_geom.dx*shear_decomp.yx/shear_decomp.yz).abs(), dst_geom.dz.abs());
        for iz in 0 .. dst_geom.nz {
            let z = dst_geom.iz2z(iz);
            for ix in 0 .. dst_geom.nx {
                let x = dst_geom.ix2x(ix);

                let taus_forw = vec![
                    -shear_decomp.yx*(x + dst_geom.dx/c2) - shear_decomp.yz*(z + dst_geom.dz/c2),
                    -shear_decomp.yx*(x + dst_geom.dx/c2) - shear_decomp.yz*(z - dst_geom.dz/c2),
                    -shear_decomp.yx*(x - dst_geom.dx/c2) - shear_decomp.yz*(z + dst_geom.dz/c2),
                    -shear_decomp.yx*(x - dst_geom.dx/c2) - shear_decomp.yz*(z - dst_geom.dz/c2),
                ];
                let forw = SplineKernel::new_trapezoid(hx_forw_y * hz_forw_y, F::one(), &taus_forw);
                forw.as_cl_bytes(&mut forw_y_buf);
            }
        }

        // forw x
        let hy_forw_x = (dst_geom.dx / shear_decomp.xy).abs();
        let hz_forw_x = fmin((dst_geom.dy*shear_decomp.xy/shear_decomp.xz).abs(), dst_geom.dz.abs());
        for iz in 0 .. dst_geom.nz {
            let z = dst_geom.iz2z(iz);
            for iy in 0 .. dst_geom.ny {
                let y = dst_geom.ix2x(iy);

                let taus_forw = vec![
                    -shear_decomp.xy*(y + dst_geom.dy/c2) - shear_decomp.xz*(z + dst_geom.dz/c2),
                    -shear_decomp.xy*(y + dst_geom.dy/c2) - shear_decomp.xz*(z - dst_geom.dz/c2),
                    -shear_decomp.xy*(y - dst_geom.dy/c2) - shear_decomp.xz*(z + dst_geom.dz/c2),
                    -shear_decomp.xy*(y - dst_geom.dy/c2) - shear_decomp.xz*(z - dst_geom.dz/c2),
                ];
                let forw = SplineKernel::new_trapezoid(hy_forw_x * hz_forw_x, F::one(), &taus_forw);
                forw.as_cl_bytes(&mut forw_x_buf);
            }
        }


        let forw_x = try!(queue.create_buffer_from_slice(&forw_x_buf));
        let forw_y = try!(queue.create_buffer_from_slice(&forw_y_buf));
        let forw_z = try!(queue.create_buffer_from_slice(&forw_z_buf));

        Ok(VolumeRotation{
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

            dst_geom_buf: dst_geom_buf,
        })
    }

    fn forw_z(self: &mut Self,
              vol: &Mem,
              out: &mut Mem,
              wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.filter_z.bind(0, &self.dst_geom_buf));
        try!(self.filter_z.bind(1, &self.spline_forw_z));
        try!(self.filter_z.bind(2, vol));
        try!(self.filter_z.bind(3, out));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_z,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    fn forw_y(self: &mut Self,
              out: &Mem,
              wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.filter_y.bind(0, &self.dst_geom_buf));
        try!(self.filter_y.bind(1, &self.spline_forw_y));
        try!(self.filter_y.bind(2, out));
        try!(self.filter_y.bind(3, &mut self.tmp));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_y,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    fn forw_x(self: &mut Self,
              out: &mut Mem,
              wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.filter_x.bind(0, &self.dst_geom_buf));
        try!(self.filter_x.bind(1, &self.spline_forw_x));
        try!(self.filter_x.bind(2, &self.tmp));
        try!(self.filter_x.bind(3, out));

        let local_size = (32, 8, 1);
        let global_size = (self.dst_geom.nx, self.dst_geom.ny, self.dst_geom.nz);

        self.queue.run_with_events(&mut self.filter_x,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    pub fn forw(self: &mut Self,
                vol: &Mem,
                out: &mut Mem,
                wait_for: &[Event]) -> Result<Event, Error> {
        let mut evt = try!(self.forw_z(vol, out, wait_for));
        evt = try!(self.forw_y(out, &[evt]));
        self.forw_x(out, &[evt])
    }

    pub fn back(self: &mut Self,
                vol: &Mem,
                out: &mut Mem,
                wait_for: &[Event]) -> Result<Event, Error> {
        unimplemented!()
    }
}

#[test]
fn test_volume_rotation() {
    use env::*;

    let rot: Rotation<f32> = Rotation::new_with_euler_angles(-1.0, 1.0, -0.0);
    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

    let src_geom = LightVolume{
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

    let rotator = VolumeRotation::new(&rot, src_geom, queue.clone()).unwrap();
}

