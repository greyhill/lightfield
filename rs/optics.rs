extern crate num;
extern crate rand;
extern crate toml;
extern crate byteorder;
use self::toml::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use serialize::*;
use cl_traits::*;
use self::byteorder::*;

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

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for Optics<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let ss = map.get("ss");
        let su = map.get("su");
        let us = map.get("us");
        let uu = map.get("uu");

        let tt = map.get("tt");
        let tv = map.get("tv");
        let vt = map.get("vt");
        let vv = map.get("vv");

        let s = map.get("s");
        let t = map.get("t");
        let u = map.get("u");
        let v = map.get("v");

        match (ss, su, us, uu,
               tt, tv, vt, vv,
               s, t, u, v) {
            (Some(&Value::Float(ss)),
             Some(&Value::Float(su)),
             Some(&Value::Float(us)),
             Some(&Value::Float(uu)),
             
             Some(&Value::Float(tt)),
             Some(&Value::Float(tv)),
             Some(&Value::Float(vt)),
             Some(&Value::Float(vv)),

             Some(&Value::Float(s)),
             Some(&Value::Float(t)),
             Some(&Value::Float(u)),
             Some(&Value::Float(v))) => Some(Optics{
                ss: F::from_f64(ss).unwrap(),
                su: F::from_f64(su).unwrap(),
                us: F::from_f64(us).unwrap(),
                uu: F::from_f64(uu).unwrap(),

                tt: F::from_f64(tt).unwrap(),
                tv: F::from_f64(tv).unwrap(),
                vt: F::from_f64(vt).unwrap(),
                vv: F::from_f64(vv).unwrap(),

                s: F::from_f64(s).unwrap(),
                t: F::from_f64(t).unwrap(),
                u: F::from_f64(u).unwrap(),
                v: F::from_f64(v).unwrap(),
            }),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("ss".to_string(), Value::Float(F::to_f64(&self.ss).unwrap()));
        tr.insert("su".to_string(), Value::Float(F::to_f64(&self.su).unwrap()));
        tr.insert("us".to_string(), Value::Float(F::to_f64(&self.us).unwrap()));
        tr.insert("uu".to_string(), Value::Float(F::to_f64(&self.uu).unwrap()));

        tr.insert("tt".to_string(), Value::Float(F::to_f64(&self.tt).unwrap()));
        tr.insert("tv".to_string(), Value::Float(F::to_f64(&self.tv).unwrap()));
        tr.insert("vt".to_string(), Value::Float(F::to_f64(&self.vt).unwrap()));
        tr.insert("vv".to_string(), Value::Float(F::to_f64(&self.vv).unwrap()));

        tr.insert("s".to_string(), Value::Float(F::to_f64(&self.s).unwrap()));
        tr.insert("t".to_string(), Value::Float(F::to_f64(&self.t).unwrap()));
        tr.insert("u".to_string(), Value::Float(F::to_f64(&self.u).unwrap()));
        tr.insert("v".to_string(), Value::Float(F::to_f64(&self.v).unwrap()));
        tr
    }
}

impl<F: Float> ClHeader for Optics<F> {
    fn header() -> &'static str {
        include_str!("../cl/optics_f32.opencl")
    }
}

impl<F: Float + ToPrimitive> ClBuffer for Optics<F> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> () {
        buf.write_f32::<LittleEndian>(F::to_f32(&self.ss).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.su).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.us).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.uu).unwrap()).unwrap();

        buf.write_f32::<LittleEndian>(F::to_f32(&self.tt).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.tv).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.vt).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.vv).unwrap()).unwrap();

        buf.write_f32::<LittleEndian>(F::to_f32(&self.s).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.t).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.u).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.v).unwrap()).unwrap();
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

#[test]
fn test_optics_serialize() {
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

    let y: Optics<f64> = Optics::from_map(&x.into_map()).unwrap();

    assert_eq!(x.ss, y.ss);
    assert_eq!(x.su, y.su);
    assert_eq!(x.us, y.us);
    assert_eq!(x.uu, y.uu);

    assert_eq!(x.tt, y.tt);
    assert_eq!(x.tv, y.tv);
    assert_eq!(x.vt, y.vt);
    assert_eq!(x.vv, y.vv);

    assert_eq!(x.s, y.s);
    assert_eq!(x.t, y.t);
    assert_eq!(x.u, y.u);
    assert_eq!(x.v, y.v);
}

