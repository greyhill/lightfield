extern crate num;
use self::num::{Float, FromPrimitive};
use bounding_geometry::*;
use occluder::*;

/// Basis functions for angles
#[derive(Clone, Debug)]
pub enum AngularBasis {
    Pillbox,
    Dirac,
}

/// Objects that can produce angular planes
pub trait AsAngularPlane<F: Float> {
    fn as_angular_plane(self: &Self, basis: AngularBasis, discretization: usize) -> AngularPlane<F>;
}

/// Angular discretization by space using pillbox basis function
#[derive(Clone, Debug)]
pub struct AngularPlane<F: Float> {
    pub ds: F,
    pub dt: F,
    pub basis: AngularBasis,
    pub s: Vec<F>,
    pub t: Vec<F>,
    pub w: Vec<F>,
}

impl<F, T> AsAngularPlane<F> for T 
where F: Float + FromPrimitive,
      T: BoundingGeometry<F> + Occluder<F> {
    fn as_angular_plane(self: &Self, basis: AngularBasis, discretization: usize) -> AngularPlane<F> {
        let geom = self.bounding_geometry(discretization, discretization);
        let mut s = Vec::new();
        let mut t = Vec::new();
        let mut w = Vec::new();

        for it in 0 .. geom.nt {
            for is in 0 .. geom.ns {
                let (s0, s1, t0, t1) = geom.pixel_bounds(is, it);
                let wk = self.rasterize(s0, s1, t0, t1, 10); // 10 provides reasonable accuracy
                if wk > F::zero() {
                    let (sk, tk) = geom.pixel_center(is, it);
                    s.push(sk);
                    t.push(tk);
                    w.push(wk);
                }
            }
        }

        AngularPlane{
            ds: geom.ds,
            dt: geom.dt,
            basis: basis,
            s: s,
            t: t,
            w: w,
        }
    }
}

