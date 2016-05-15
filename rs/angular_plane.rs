extern crate num;
use self::num::{Float, FromPrimitive};
use bounding_geometry::*;
use occluder::*;

/// Objects that can produce angular planes
pub trait AngularPlane<T> {
    fn as_angular_plane(self: &Self, discretization: usize) -> T;
}

/// Angular discretization by space using pillbox basis function
#[derive(Clone, Debug)]
pub struct SpatialPillbox<F: Float> {
    pub ds: F,
    pub dt: F,
    pub s: Vec<F>,
    pub t: Vec<F>,
    pub w: Vec<F>,
}

/// Angular discretization by space using dirac basis function
#[derive(Clone, Debug)]
pub struct SpatialDirac<F: Float> {
    pub s: Vec<F>,
    pub t: Vec<F>,
    pub w: Vec<F>,
}

impl<F, T> AngularPlane<SpatialPillbox<F>> for T 
where F: Float + FromPrimitive,
      T: BoundingGeometry<F> + Occluder<F> {
    fn as_angular_plane(self: &Self, discretization: usize) -> SpatialPillbox<F> {
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

        SpatialPillbox{
            ds: geom.ds,
            dt: geom.dt,
            s: s,
            t: t,
            w: w,
        }
    }
}

impl<F, T> AngularPlane<SpatialDirac<F>> for T
where F: Float + FromPrimitive,
      T: BoundingGeometry<F> + Occluder<F> {
    fn as_angular_plane(self: &Self, discretization: usize) -> SpatialDirac<F> {
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
                    w.push(wk * geom.ds * geom.dt);
                }
            }
        }

        SpatialDirac{
            s: s,
            t: t,
            w: w,
        }
    }
}

