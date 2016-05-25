extern crate num;
extern crate proust;

use light_volume::*;
use self::num::{Float, FromPrimitive, ToPrimitive};

pub struct ResampleVolume<'src, 'dst, F: 'src + 'dst + Float> {
    pub src: &'src LightVolume<F>,
    pub dst: &'dst LightVolume<F>,
}

