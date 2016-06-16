extern crate num;
extern crate proust;
use self::num::{Float, ToPrimitive, FromPrimitive};
use self::proust::*;
use std::marker::PhantomData;
use cl_traits::*;

/// Common vector operations
pub struct VectorMath<F: Float + ToPrimitive + FromPrimitive> {
    queue: CommandQueue,
    set: Kernel,
    mix: Kernel,
    m_: PhantomData<F>,
}

impl<F: Float + ToPrimitive + FromPrimitive> ClHeader for VectorMath<F> {
    fn header() -> &'static str {
        include_str!("../cl/vector_math_f32.opencl")
    }
}

impl<F: Float + ToPrimitive + FromPrimitive> VectorMath<F> {
    pub fn new(queue: CommandQueue) -> Result<Self, Error> {
        // get OpenCL objects
        let context = try!(queue.context());
        let device = try!(queue.device());

        // build program
        let sources = &[Self::header()];
        let unbuilt = try!(Program::new_from_source(context, sources));
        let built = try!(unbuilt.build(&[device]));

        // create kernels
        let set = try!(built.create_kernel("VectorMath_set"));
        let mix = try!(built.create_kernel("VectorMath_mix"));

        Ok(VectorMath{
            queue: queue,
            set: set,
            mix: mix,
            m_: PhantomData,
        })
    }

    /// Implements `vec[i] = val`
    pub fn set(self: &mut Self,
               np: usize,
               vec: &mut Mem,
               val: F,
               wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.set.bind_scalar(0, &(np as i32)));
        try!(self.set.bind_mut(1, vec));
        try!(self.set.bind_scalar(2, &F::to_f32(&val).unwrap()));

        let local_size = (256, 1, 1);
        let global_size = (np, 1, 1);

        self.queue.run_with_events(&mut self.set,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    /// Implements `out[i] = ax*x[i] + ay*y[i]`
    pub fn mix(self: &mut Self,
               np: usize,
               x: &Mem,
               y: &Mem,
               ax: F,
               ay: F,
               out: &mut Mem,
               wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.mix.bind_scalar(0, &(np as i32)));
        try!(self.mix.bind(1, x));
        try!(self.mix.bind(2, y));
        try!(self.mix.bind_scalar(3, &F::to_f32(&ax).unwrap()));
        try!(self.mix.bind_scalar(4, &F::to_f32(&ay).unwrap()));
        try!(self.mix.bind_mut(5, out));

        let local_size = (256, 1, 1);
        let global_size = (np, 1, 1);

        self.queue.run_with_events(&mut self.mix,
                                   local_size,
                                   global_size,
                                   wait_for)
    }
}

