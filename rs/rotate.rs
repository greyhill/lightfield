extern crate num;
extern crate toml;
extern crate proust;

use env::*;
use cl_traits::*;
use self::num::Float;
use self::proust::*;
use std::marker::PhantomData;
use image_geom::*;

pub struct Rotate<F: Float> {
    queue: CommandQueue,
    forw_kernel: Kernel,
    back_kernel: Kernel,

    _x: PhantomData<F>,
}

impl Rotate<f32> {
    pub fn new(queue: CommandQueue,
               geom: &ImageGeometry<f32>) -> Result<Self, Error> {
        let ctx = try!(queue.context());
        let device = try!(queue.device());
        let source = [
            geom.header("geom"),
            include_str!("../cl/rotate_f32.opencl").to_string(),
        ];
        let unbuilt = try!(Program::new_from_source(ctx.clone(), &source));
        let built = try!(unbuilt.build(&[device]));
        let forw_kernel = try!(built.create_kernel("rotate_forw"));
        let back_kernel = try!(built.create_kernel("rotate_back"));
        Ok(Rotate{
            queue: queue,
            forw_kernel: forw_kernel,
            back_kernel: back_kernel,
            _x: PhantomData,
        })
    }

    pub fn forw(self: &Self,
                s0: f32, s1: f32,
                t0: f32, t1: f32,
                src: &Mem, dst: &mut Mem,
                wait_for: &[Event]) -> Result<Event, Error> {
        let mut k = self.forw_kernel.clone();

        try!(k.bind_scalar(0, &s0));
        try!(k.bind_scalar(1, &s1));
        try!(k.bind_scalar(2, &t0));
        try!(k.bind_scalar(3, &t1));
        try!(k.bind(4, src));
        try!(k.bind_mut(5, dst));

        let local_size = (32, 8, 1);
        unimplemented!()
    }

    pub fn back(self: &Self,
                s0: f32, s1: f32,
                t0: f32, t1: f32,
                src: &Mem, dst: &mut Mem,
                wait_for: &[Event]) -> Result<Event, Error> {
        unimplemented!()
    }
}

#[test]
fn test_rotate_f32_build() {
    let env = Environment::new_easy().unwrap();
    let geom = ImageGeometry::<f32>{
        ns: 1024,
        nt: 1024,
        ds: 1.0,
        dt: 1.0,
        offset_s: 0.0,
        offset_t: 0.0,
    };
    let rot = Rotate::new(env.queues[0].clone(), &geom).unwrap();
}

