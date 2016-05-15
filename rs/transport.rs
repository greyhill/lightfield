extern crate num;
extern crate proust;

use env::*;
use cl_traits::*;
use self::num::Float;
use self::proust::*;
use std::marker::PhantomData;

pub struct Transport<F: Float> {
    queue: CommandQueue,
    forw_kernel_s: Kernel,
    forw_kernel_t: Kernel,
    back_kernel_s: Kernel,
    back_kernel_t: Kernel,

    _x: PhantomData<F>,
}

impl Transport<f32> {
}

