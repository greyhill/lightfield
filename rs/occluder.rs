extern crate num;
use self::num::{Float, FromPrimitive};
use image_geom::*;

/// A light-occluding object
pub trait Occluder<F: Float + FromPrimitive> {
    /// Returns `true` if the object occludes at this point, `false` if it does not
    fn occludes(self: &Self, s: F, t: F) -> bool;

    /// Returns fraction of the rectangular region not occluded by this object
    ///
    /// Complete occlusion is 0, partial occlusion between 0 and 1, and no
    /// occlusion is 1.
    fn rasterize(self: &Self, s0: F, s1: F, t0: F, t1: F, discretization: usize) -> F {
        let mut tr = F::zero();
        let mut denom = F::zero();
        for it in 0 .. discretization {
            let t = t0 + (t1 - t0) * F::from_usize(it).unwrap() / F::from_usize(discretization - 1).unwrap();
            for is in 0 .. discretization {
                let s = s0 + (s1 - s0) * F::from_usize(is).unwrap() / F::from_usize(discretization - 1).unwrap();
                if self.occludes(s, t) {
                    tr = tr + F::one();
                }
                denom = denom + F::one();
            }
        }
        tr / denom
    }

    /// Rasterize every pixel in the given geometry
    fn rasterize_geom(self: &Self, geom: &ImageGeometry<F>, discretization: usize) -> Vec<F> {
        let mut tr = Vec::with_capacity(geom.ns * geom.nt);
        for it in 0 .. geom.nt {
            for is in 0 .. geom.ns {
                let (s0, s1, t0, t1) = geom.pixel_bounds(is, it);
                tr.push(self.rasterize(s0, s1, t0, t1, discretization));
            }
        }
        tr
    }
}

