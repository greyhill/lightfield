extern crate num;
extern crate rand;
extern crate proust;
use self::num::{Float, FromPrimitive};
use self::proust::*;
use self::rand::{thread_rng, Rng};
use std::path::Path;

fn reduce_shape(shape: &[usize]) -> usize {
    let mut tr = 1;
    for s in shape.iter() {
        tr *= *s;
    }
    tr
}

/// Trait for types describing geometries
pub trait Geometry<F: Float + FromPrimitive> {
    /// Returns the dimensions of elements of this type
    fn shape(self: &Self) -> Vec<usize>;

    /// Returns a buffer of ones
    fn ones(self: &Self) -> Vec<F> {
        let np = reduce_shape(&self.shape());
        vec![F::one(); np]
    }

    /// Returns a buffer of zeros
    fn zeros(self: &Self) -> Vec<F> {
        let np = reduce_shape(&self.shape());
        vec![F::zero(); np]
    }

    /// Returns a buffer of random values
    fn rands(self: &Self) -> Vec<F> {
        let np = reduce_shape(&self.shape());
        let mut tr = Vec::with_capacity(np);
        for _ in 0 .. np {
            tr.push(F::from_f64(thread_rng().next_f64()).unwrap());
        }
        tr
    }

    /// Returns an OpenCL buffer of ones
    fn ones_buf(self: &Self, queue: &CommandQueue) -> Result<Mem, Error> {
        queue.create_buffer_from_slice(&self.ones())
    }

    /// Returns an OpenCL buffer of zeros
    fn zeros_buf(self: &Self, queue: &CommandQueue) -> Result<Mem, Error> {
        queue.create_buffer_from_slice(&self.zeros())
    }

    /// Returns an OpenCL buffer of random values
    fn rands_buf(self: &Self, queue: &CommandQueue) -> Result<Mem, Error> {
        queue.create_buffer_from_slice(&self.rands())
    }

    /// Save a buffer to a path
    fn save<P: AsRef<Path>>(self: &Self, buf: &[F], path: P) -> Result<(), ()>;

    /// Load a buffer from a path
    fn load<P: AsRef<Path>>(self: &Self, path: P) -> Result<Vec<F>, ()>;
}
