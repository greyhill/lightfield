use env::*;
use optics::*;

use std::ptr;

/// C API's optical transformation object
#[repr(C)]
#[allow(non_snake_case)]
pub struct LFOpticalX {
    ss: f32,
    us: f32,
    su: f32,
    uu: f32,

    tt: f32,
    vt: f32,
    tv: f32,
    vv: f32,

    s: f32,
    t: f32,
    u: f32,
    v: f32,
}

impl LFOpticalX {
    fn as_optics(self: &Self) -> Optics<f32> {
        Optics{
            ss: self.ss,
            su: self.su,
            us: self.us,
            uu: self.uu,

            tt: self.tt,
            tv: self.tv,
            vt: self.vt,
            vv: self.vv,

            s: self.s,
            t: self.t,
            u: self.u,
            v: self.v,
        }
    }

    fn from_optics(o: &Optics<f32>) -> Self {
        LFOpticalX{
            ss: o.ss,
            su: o.su,
            us: o.us,
            uu: o.uu,

            tt: o.tt,
            tv: o.tv,
            vt: o.vt,
            vv: o.vv,

            s: o.s,
            t: o.t,
            u: o.u,
            v: o.v,
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_identity(x: *mut LFOpticalX) {
    *x = LFOpticalX::from_optics(&Optics::identity());
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_compose(lhs: *const LFOpticalX,
                                 rhs: *const LFOpticalX,
                                 out: *mut LFOpticalX) {
    *out = LFOpticalX::from_optics(
        &(*lhs).as_optics().compose(&(*rhs).as_optics())
    );
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_translation(x: *mut LFOpticalX,
                                     distance: f32) {
    *x = LFOpticalX::from_optics(&Optics::translation(&distance));
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_lens(x: *mut LFOpticalX,
                              center_x: f32,
                              center_y: f32,
                              focal_length: f32) {
    *x = LFOpticalX::from_optics(&Optics::lens(&center_x,
                                               &center_y,
                                               &focal_length));
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_invert(x: *const LFOpticalX,
                                out: *mut LFOpticalX) {
    *out = LFOpticalX::from_optics(&(*x).as_optics().invert());
}

#[no_mangle]
#[allow(non_snake_case)]
/// C API to create a new `Environment` object
pub unsafe fn LFEnvironment_new() -> *mut Environment {
    match Environment::new_easy() {
        Ok(env) => {
            Box::into_raw(Box::new(env))
        },
        Err(e) => {
            println!("Error creating environment: {:?}", e);
            ptr::null_mut()
        },
    }
}

#[no_mangle]
#[allow(non_snake_case)]
/// C API to destroy an `Environment` object
pub unsafe fn LFEnvironment_del(env: *mut Environment) {
    Box::from_raw(env);
}

