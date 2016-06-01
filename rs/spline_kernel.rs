extern crate num;
extern crate byteorder;
use self::num::{Float};
use cl_traits::*;
use self::byteorder::*;

/// Kernel of a Toeplitz-like operation
#[derive(Clone, Debug)]
pub enum SplineKernel<F: Float> {
    Rect(F, F, [F; 2]),
    Trapezoid(F, F, [F; 4]),
}

impl<F: Float> SplineKernel<F> {
    /// Height of the kernel
    pub fn height(self: &Self) -> F {
        match self {
            &SplineKernel::Rect(ref height, _, _) => height.clone(),
            &SplineKernel::Trapezoid(ref height, _, _) => height.clone(),
        }
    }

    /// Magnification of the input coordinate
    pub fn magnification(self: &Self) -> F {
        match self {
            &SplineKernel::Rect(_, ref mag, _) => mag.clone(),
            &SplineKernel::Trapezoid(_, ref mag, _) => mag.clone(),
        }
    }
}

impl ClHeader for SplineKernel<f32> {
    fn header() -> &'static str {
        include_str!("../cl/spline_kernel_f32.opencl")
    }
}

impl<> ClBuffer for SplineKernel<f32> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> () {
        match self {
            &SplineKernel::Rect(ref height, ref mag, ref taus) => {
                buf.write_f32::<LittleEndian>(*height).unwrap();
                buf.write_f32::<LittleEndian>(*mag).unwrap();
                buf.write_f32::<LittleEndian>(taus[0]).unwrap();
                buf.write_f32::<LittleEndian>(taus[1]).unwrap();
            },
            &SplineKernel::Trapezoid(ref height, ref mag, ref taus) => {
                buf.write_f32::<LittleEndian>(*height).unwrap();
                buf.write_f32::<LittleEndian>(*mag).unwrap();
                buf.write_f32::<LittleEndian>(taus[0]).unwrap();
                buf.write_f32::<LittleEndian>(taus[1]).unwrap();
                buf.write_f32::<LittleEndian>(taus[2]).unwrap();
                buf.write_f32::<LittleEndian>(taus[3]).unwrap();
            },
        }
    }
}

