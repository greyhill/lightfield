extern crate num;
extern crate byteorder;
use self::num::{Float, ToPrimitive};
use cl_traits::*;
use self::byteorder::*;

/// Kernel of a Toeplitz-like operation
#[derive(Clone, Debug)]
pub enum SplineKernel<F: Float> {
    Rect(F, F, [F; 2]),
    Trapezoid(F, F, [F; 4]),
    Quad(F, F, [F; 8]),
}

impl<F: Float> SplineKernel<F> {
    /// Sorts the taus and returns a SplineKernel
    pub fn new_trapezoid(height: F, mag: F, taus: &[F]) -> Self {
        let mut t = taus.to_owned();
        t.sort_by(|l, r| l.partial_cmp(r).unwrap());
        SplineKernel::Trapezoid(height, mag, [taus[0], taus[1], taus[2], taus[3]])
    }

    /// sorts taus and returns a SplineKernel
    pub fn new_quad(height: F, mag: F, taus: &[F]) -> Self {
        let mut t = taus.to_owned();
        t.sort_by(|l, r| l.partial_cmp(r).unwrap());
        SplineKernel::Quad(height, mag, [taus[0], taus[1], taus[2], taus[3],
                                         taus[4], taus[5], taus[6], taus[7]])
    }

    /// Height of the kernel
    pub fn height(self: &Self) -> F {
        match self {
            &SplineKernel::Rect(ref height, _, _) => height.clone(),
            &SplineKernel::Trapezoid(ref height, _, _) => height.clone(),
            &SplineKernel::Quad(ref height, _, _) => height.clone(),
        }
    }

    /// Magnification of the input coordinate
    pub fn magnification(self: &Self) -> F {
        match self {
            &SplineKernel::Rect(_, ref mag, _) => mag.clone(),
            &SplineKernel::Trapezoid(_, ref mag, _) => mag.clone(),
            &SplineKernel::Quad(_, ref mag, _) => mag.clone(),
        }
    }
}

impl<F: Float> ClHeader for SplineKernel<F> {
    fn header() -> &'static str {
        include_str!("../cl/spline_kernel_f32.opencl")
    }
}

impl<F: Float + ToPrimitive> ClBuffer for SplineKernel<F> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> () {
        match self {
            &SplineKernel::Rect(ref height, ref mag, ref taus) => {
                buf.write_f32::<LittleEndian>(F::to_f32(height).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(mag).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[0]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[1]).unwrap()).unwrap();
            },
            &SplineKernel::Trapezoid(ref height, ref mag, ref taus) => {
                buf.write_f32::<LittleEndian>(F::to_f32(height).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(mag).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[0]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[1]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[2]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[3]).unwrap()).unwrap();
            },
            &SplineKernel::Quad(ref height, ref mag, ref taus) => {
                buf.write_f32::<LittleEndian>(F::to_f32(height).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(mag).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[0]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[1]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[2]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[3]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[4]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[5]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[6]).unwrap()).unwrap();
                buf.write_f32::<LittleEndian>(F::to_f32(&taus[7]).unwrap()).unwrap();
            },
        }
    }
}

