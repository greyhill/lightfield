extern crate num;
use self::num::{Float, FromPrimitive};
use image_geom::*;
use angular_plane::*;
use optics::*;
use spline_kernel::*;
use std::mem::swap;

/// One plane in a light transport stack
#[derive(Clone, Debug)]
pub struct LightFieldGeometry<F: Float> {
    pub geom: ImageGeometry<F>,
    pub plane: AngularPlane<F>,
    pub to_plane: Optics<F>,
}

impl<F: Float + FromPrimitive> LightFieldGeometry<F> {
    /// Returns the 4d "squared volume" of a pixel on the light field
    ///
    /// This expression comes from integrating the spatial and angular basis
    /// functions over their domain; see Table 1 in the PDF documentation.
    pub fn pixel_volume(self: &Self) -> F {
        match &self.plane.basis {
            &AngularBasis::Dirac => {
                let vs = (self.plane.ds / self.to_plane.su).abs() * self.geom.ds;
                let vt = (self.plane.dt / self.to_plane.tv).abs() * self.geom.dt;
                (vs * vt).abs()
            },
            &AngularBasis::Pillbox => {
                let c2 = F::from_f32(2f32).unwrap();

                let s_m_a = self.plane.ds / c2 / self.to_plane.su.abs();
                let s_m_b = self.geom.ds / c2 * (self.to_plane.ss / self.to_plane.su).abs();
                let s_m = s_m_a.max(s_m_b);
                let s_h = self.geom.ds.min(self.plane.ds / self.to_plane.ss.abs());

                let t_m_a = self.plane.dt / c2 / self.to_plane.tv.abs();
                let t_m_b = self.geom.dt / c2 * (self.to_plane.tt / self.to_plane.tv).abs();
                let t_m = t_m_a.max(t_m_b);
                let t_h = self.geom.dt.min(self.plane.dt / self.to_plane.tt.abs());

                (c2 * c2 * s_m * s_h * t_m * t_h).abs()
            },
        }
    }

    /// Returns the area of the pixel
    pub fn pixel_area(self: &Self) -> F {
        self.plane.ds * self.plane.dt
    }

    /// Returns an optical transformation from this geometry to another
    ///
    /// This assumes both geometies are using the same optical plane.
    pub fn optics_to(self: &Self, dst: &LightFieldGeometry<F>) -> Optics<F> {
        dst.to_plane.invert().compose(&self.to_plane)
    }

    /// Returns two `SplineKernel`s describing the rows of a light transport matrix
    ///
    /// The first `SplineKernel` describes the `s` filtering matrix and the second
    /// one describes the `t` filtering matrix.
    ///
    /// This is mostly a utility function used by other transport objects.
    pub fn transport_to(self: &Self, dst: &LightFieldGeometry<F>, ia: usize) -> (SplineKernel<F>, SplineKernel<F>) {
        let src_to_dst = self.optics_to(dst);
        match (&self.plane.basis, &dst.plane.basis) {
            (&AngularBasis::Dirac, &AngularBasis::Dirac) => {
                let xs = self.transport_s_dirac(dst, &src_to_dst, ia);
                let xt = self.transport_t_dirac(dst, &src_to_dst, ia);
                (xs, xt)
            },
            (&AngularBasis::Pillbox, &AngularBasis::Pillbox) => {
                let xs = self.transport_s_pillbox(dst, &src_to_dst, ia);
                let xt = self.transport_t_pillbox(dst, &src_to_dst, ia);
                (xs, xt)
            },
            _ => {
                panic!("Cannot transport between mismatched angular basis functions");
            },
        }
    }

    fn transport_s_dirac(self: &Self, dst: &LightFieldGeometry<F>, 
                         src2dst: &Optics<F>, ia: usize) -> SplineKernel<F> {
        let plane = &self.plane;
        let s = plane.s[ia];
        let c2 = F::one() + F::one();
        let src2root = &self.to_plane;

        let alpha = src2dst.ss - src2root.ss*src2dst.su/src2root.su;
        let beta = src2dst.s + src2dst.su*(s - src2root.s)/src2root.su;
        let h = (plane.ds / src2root.su).abs();

        let mut tau0 = (-dst.geom.ds/c2 - beta)/alpha;
        let mut tau1 = (dst.geom.ds/c2 - beta)/alpha;
        if tau0 > tau1  {
            swap(&mut tau0, &mut tau1);
        }

        let mag = F::one() / alpha;

        SplineKernel::Rect(h, mag, [tau0, tau1])
    }

    fn transport_t_dirac(self: &Self, dst: &LightFieldGeometry<F>, 
                         src2dst: &Optics<F>, ia: usize) -> SplineKernel<F> {
        let plane = &self.plane;
        let t = plane.t[ia];
        let c2 = F::one() + F::one();
        let src2root = &self.to_plane;

        let alpha = src2dst.tt - src2root.tt*src2dst.tv/src2root.tv;
        let beta = src2dst.t + src2dst.tv*(t - src2root.t)/src2root.tv;
        let h = (plane.dt / src2root.tv).abs();

        let mut tau0 = (-dst.geom.dt/c2 - beta)/alpha;
        let mut tau1 = (dst.geom.dt/c2 - beta)/alpha;
        if tau0 > tau1  {
            swap(&mut tau0, &mut tau1);
        }

        let mag = F::one() / alpha;

        SplineKernel::Rect(h, mag, [tau0, tau1])
    }

    fn transport_s_pillbox(self: &Self, dst: &LightFieldGeometry<F>, 
                           src2dst: &Optics<F>, ia: usize) -> SplineKernel<F> {
        let plane = &self.plane;
        let s = plane.s[ia];
        let c2 = F::one() + F::one();
        let src2root = &self.to_plane;

        let alpha = src2dst.ss - src2dst.su*src2root.ss/src2root.su;
        let beta = src2dst.su / src2root.su;
        let gamma = src2dst.s - src2dst.su*src2root.s/src2root.su;
        let h = (plane.ds / src2root.su).abs().min((dst.geom.ds / src2dst.su).abs());

        let mut taus = vec![
            (dst.geom.ds/c2 - beta*(s + plane.ds/c2) - gamma)/alpha,
            (dst.geom.ds/c2 - beta*(s - plane.ds/c2) - gamma)/alpha,
            (-dst.geom.ds/c2 - beta*(s + plane.ds/c2) - gamma)/alpha,
            (-dst.geom.ds/c2 - beta*(s - plane.ds/c2) - gamma)/alpha,
        ];
        taus.sort_by(|l,r| l.partial_cmp(r).unwrap());
        let taus_array = [
            taus[0], taus[1], taus[2], taus[3]
        ];

        let mag = F::one() / alpha;

        SplineKernel::Trapezoid(h, mag, taus_array)
    }

    fn transport_t_pillbox(self: &Self, dst: &LightFieldGeometry<F>, 
                           src2dst: &Optics<F>, ia: usize) -> SplineKernel<F> {
        let plane = &self.plane;
        let t = plane.t[ia];
        let c2 = F::one() + F::one();
        let src2root = &self.to_plane;

        let alpha = src2dst.tt - src2dst.tv*src2root.tt/src2root.tv;
        let beta = src2dst.tv / src2root.tv;
        let gamma = src2dst.t - src2dst.tv*src2root.t/src2root.tv;
        let h = (plane.dt / src2root.tv).abs().min((dst.geom.dt / src2dst.tv).abs());

        let mut taus = vec![
            (dst.geom.dt/c2 - beta*(t + plane.dt/c2) - gamma)/alpha,
            (dst.geom.dt/c2 - beta*(t - plane.dt/c2) - gamma)/alpha,
            (-dst.geom.dt/c2 - beta*(t + plane.dt/c2) - gamma)/alpha,
            (-dst.geom.dt/c2 - beta*(t - plane.dt/c2) - gamma)/alpha,
        ];
        taus.sort_by(|l,r| l.partial_cmp(r).unwrap());
        let taus_array = [
            taus[0], taus[1], taus[2], taus[3]
        ];

        let mag = F::one() / alpha;

        SplineKernel::Trapezoid(h, mag, taus_array)
    }
}

