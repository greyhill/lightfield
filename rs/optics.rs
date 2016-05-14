extern crate num;
extern crate rand;
use self::num::Float;

/// Affine optical transformation for light transport
#[derive(Clone, Debug)]
pub struct Optics<F: Float> {
    pub ss: F,
    pub su: F,
    pub us: F,
    pub uu: F,

    pub tt: F,
    pub tv: F,
    pub vt: F,
    pub vv: F,

    pub s: F,
    pub t: F,
    pub u: F,
    pub v: F,
}

impl<F: Float> Optics<F> {
    /// Optical transformation that does nothing
    pub fn identity() -> Self {
        Optics{ 
            ss: F::one(),
            su: F::zero(),
            us: F::zero(),
            uu: F::one(),

            tt: F::one(),
            tv: F::zero(),
            vt: F::zero(),
            vv: F::one(),

            s: F::zero(),
            t: F::zero(),
            u: F::zero(),
            v: F::zero(),
        }
    }

    /// Translation normal to the optical plane
    pub fn translation(dist: &F) -> Self {
        Optics{
            ss: F::one(),
            su: dist.clone(),
            us: F::zero(),
            uu: F::one(),

            tt: F::one(),
            tv: dist.clone(),
            vt: F::zero(),
            vv: F::one(),

            s: F::zero(),
            t: F::zero(),
            u: F::zero(),
            v: F::zero(),
        }
    }

    /// Thin lens optics
    pub fn lens(center_s: &F, center_t: &F, focal_length: &F) -> Self {
        Optics{
            ss: F::one(),
            su: F::zero(),
            us: - F::one() / focal_length.clone(),
            uu: F::one(),

            tt: F::one(),
            tv: F::zero(),
            vt: - F::one() / focal_length.clone(),
            vv: F::one(),

            s: F::zero(),
            t: F::zero(),
            u: center_s.clone() / focal_length.clone(),
            v: center_t.clone() / focal_length.clone(),
        }
    }

    /// Anisotropic thin lens
    pub fn anisotropic_lens(center_s: &F, center_t: &F, 
                            focal_length_s: &F, focal_length_t: &F) -> Self {
        Optics{
            ss: F::one(),
            su: F::zero(),
            us: - F::one() / focal_length_s.clone(),
            uu: F::one(),

            tt: F::one(),
            tv: F::zero(),
            vt: - F::one() / focal_length_t.clone(),
            vv: F::one(),

            s: F::zero(),
            t: F::zero(),
            u: center_s.clone() / focal_length_s.clone(),
            v: center_t.clone() / focal_length_t.clone(),
        }
    }

    /// Returns the inverse of this transformation
    pub fn invert(self: &Self) -> Self {
        let s_det = self.ss*self.uu - self.su*self.us;
        let ss = self.uu / s_det;
        let su = - self.su / s_det;
        let us = - self.us / s_det;
        let uu = self.ss / s_det;
        let s = -(ss*self.s + su*self.u);
        let u = -(us*self.s + uu*self.u);

        let t_det = self.tt*self.vv - self.tv*self.vt;
        let tt = self.vv / t_det;
        let tv = - self.tv / t_det;
        let vt = - self.vt / t_det;
        let vv = self.tt / t_det;
        let t = -(tt*self.t + tv*self.v);
        let v = -(vt*self.t + vv*self.v);

        Optics{
            ss: ss,
            su: su,
            us: us,
            uu: uu,

            tt: tt,
            tv: tv,
            vt: vt,
            vv: vv,

            s: s,
            t: t,
            u: u,
            v: v,
        }
    }

    /// Apply this transformation after the given one
    pub fn compose(self: &Self, rhs: &Self) -> Self {
        let ss = self.ss*rhs.ss + self.su*rhs.us;
        let su = self.ss*rhs.su + self.su*rhs.uu;
        let us = self.us*rhs.ss + self.uu*rhs.us;
        let uu = self.us*rhs.su + self.uu*rhs.uu;

        let tt = self.tt*rhs.tt + self.tv*rhs.vt;
        let tv = self.tt*rhs.tv + self.tv*rhs.vv;
        let vt = self.vt*rhs.tt + self.vv*rhs.vt;
        let vv = self.vt*rhs.tv + self.vv*rhs.vv;

        let s = self.s + self.ss*rhs.s + self.su*rhs.u;
        let u = self.u + self.us*rhs.s + self.uu*rhs.u;
        let t = self.t + self.tt*rhs.t + self.tv*rhs.v;
        let v = self.v + self.vt*rhs.t + self.vv*rhs.v;
    
        Optics{
            ss: ss,
            su: su,
            us: us,
            uu: uu,

            tt: tt,
            tv: tv,
            vt: vt,
            vv: vv,

            s: s,
            t: t,
            u: u,
            v: v,
        }
    }

    /// Apply this transformation after the given one
    pub fn then(self: &Self, then: &Self) -> Self {
        then.compose(self)
    }

    /// Returns the (s,t) focused distances
    pub fn focused_distance(self: &Self) -> (F, F) {
        let focused_s = - self.su / self.uu;
        let focused_t = - self.tv / self.vv;
        (focused_s, focused_t)
    }
}

#[test]
fn test_cam() {
    use self::rand::*;

    let d_det_lens = thread_rng().next_f64().abs();
    let d_lens_scene = thread_rng().next_f64().abs();
    let c_s = thread_rng().next_f64();
    let c_t = thread_rng().next_f64();
    let f_s = thread_rng().next_f64().abs();
    let f_t = thread_rng().next_f64().abs();

    let x = Optics::translation(&d_det_lens)
                .then(&Optics::anisotropic_lens(&c_s, &c_t, &f_s, &f_t))
                .then(&Optics::translation(&d_lens_scene));
    let xi = x.invert();
    let xix = xi.compose(&x);

    // should be almost identity
    assert!((xix.ss - 1f64).abs() < 1e-8);
    assert!((xix.su - 0f64).abs() < 1e-8);
    assert!((xix.us - 0f64).abs() < 1e-8);
    assert!((xix.uu - 1f64).abs() < 1e-8);
    assert!((xix.s - 0f64).abs() < 1e-8);
    assert!((xix.u - 0f64).abs() < 1e-8);

    assert!((xix.tt - 1f64).abs() < 1e-8);
    assert!((xix.tv - 0f64).abs() < 1e-8);
    assert!((xix.vt - 0f64).abs() < 1e-8);
    assert!((xix.vv - 1f64).abs() < 1e-8);
    assert!((xix.t - 0f64).abs() < 1e-8);
    assert!((xix.v - 0f64).abs() < 1e-8);
}

#[test]
fn test_ulens() {
    use self::rand::*;

    let d_det_array = thread_rng().next_f64().abs();
    let d_array_lens = thread_rng().next_f64().abs();
    let d_lens_scene = thread_rng().next_f64().abs();

    let cu_s = thread_rng().next_f64();
    let cu_t = thread_rng().next_f64();
    let fu_s = thread_rng().next_f64().abs();
    let fu_t = thread_rng().next_f64().abs();

    let c_s = thread_rng().next_f64();
    let c_t = thread_rng().next_f64();
    let f_s = thread_rng().next_f64().abs();
    let f_t = thread_rng().next_f64().abs();

    let x = Optics::translation(&d_det_array)
                .then(&Optics::anisotropic_lens(&cu_s, &cu_t, &fu_s, &fu_t))
                .then(&Optics::translation(&d_array_lens))
                .then(&Optics::anisotropic_lens(&c_s, &c_t, &f_s, &f_t))
                .then(&Optics::translation(&d_lens_scene));
    let xi = x.invert();
    let xix = xi.compose(&x);

    // should be almost identity
    assert!((xix.ss - 1f64).abs() < 1e-8);
    assert!((xix.su - 0f64).abs() < 1e-8);
    assert!((xix.us - 0f64).abs() < 1e-8);
    assert!((xix.uu - 1f64).abs() < 1e-8);
    assert!((xix.s - 0f64).abs() < 1e-8);
    assert!((xix.u - 0f64).abs() < 1e-8);

    assert!((xix.tt - 1f64).abs() < 1e-8);
    assert!((xix.tv - 0f64).abs() < 1e-8);
    assert!((xix.vt - 0f64).abs() < 1e-8);
    assert!((xix.vv - 1f64).abs() < 1e-8);
    assert!((xix.t - 0f64).abs() < 1e-8);
    assert!((xix.v - 0f64).abs() < 1e-8);
}

