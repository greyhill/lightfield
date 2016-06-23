extern crate num;
use self::num::Float;
use image_geom::*;

/// 2d objects that can be bounded by rectangular regions
pub trait BoundingGeometry<F: Float> {
    fn bounding_geometry(self: &Self, ns: usize, nt: usize) -> ImageGeometry<F>;
}
