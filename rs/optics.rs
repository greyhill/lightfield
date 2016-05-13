extern crate num;
use self::num::Float;

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
}
