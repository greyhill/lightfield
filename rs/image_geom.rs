extern crate num;
use self::num::{Float};

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

