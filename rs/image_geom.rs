extern crate num;
use cl_traits::*;
use self::num::{Float, FromPrimitive};

/// Pixel or plane geometry
#[derive(Clone, Debug)]
pub struct ImageGeometry<F: Float> {
    pub ns: usize,
    pub nt: usize,
    pub ds: F,
    pub dt: F,
    pub offset_s: F,
    pub offset_t: F,
}

impl<F: Float + FromPrimitive> ImageGeometry<F> {
    pub fn ws(self: &Self) -> F {
        (F::from_usize(self.ns).unwrap() - F::one())/F::from_f32(2f32).unwrap() + self.offset_s
    }

    pub fn wt(self: &Self) -> F {
        (F::from_usize(self.nt).unwrap() - F::one())/F::from_f32(2f32).unwrap() + self.offset_t
    }
}

impl ClHeader for ImageGeometry<f32> {
    fn header<S: AsRef<str>>(self: &Self, name: S) -> String {
        format!(include_str!("../cl/image_geom_f32.opencl"),
                             name = name.as_ref(),
                             ns = self.ns,
                             nt = self.nt,
                             ds = self.ds,
                             dt = self.dt,
                             offset_s = self.offset_s,
                             offset_t = self.offset_t,
                             ws = self.ws(),
                             wt = self.wt()).to_string()
    }
}

