use env::*;
use optics::*;
use transport::*;
use image_geom::*;
use angular_plane::*;
use light_field_geom::*;
use geom::*;

use std::ptr;
use std::slice;

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
    fn as_rust(self: &Self) -> Optics<f32> {
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

    fn from_rust(o: &Optics<f32>) -> Self {
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
    *x = LFOpticalX::from_rust(&Optics::identity());
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_compose(lhs: *const LFOpticalX,
                                 rhs: *const LFOpticalX,
                                 out: *mut LFOpticalX) {
    *out = LFOpticalX::from_rust(
        &(*lhs).as_rust().compose(&(*rhs).as_rust())
    );
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_translation(x: *mut LFOpticalX,
                                     distance: f32) {
    *x = LFOpticalX::from_rust(&Optics::translation(&distance));
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_lens(x: *mut LFOpticalX,
                              center_x: f32,
                              center_y: f32,
                              focal_length: f32) {
    *x = LFOpticalX::from_rust(&Optics::lens(&center_x,
                                               &center_y,
                                               &focal_length));
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFOpticalX_invert(x: *const LFOpticalX,
                                out: *mut LFOpticalX) {
    *out = LFOpticalX::from_rust(&(*x).as_rust().invert());
}

#[repr(C)]
#[allow(non_snake_case)]
/// C API angular plane representation
pub struct LFAngularPlane {
    ds: f32,
    dt: f32,
    basis: u32,

    num_points: usize,
    points_s: *const f32,
    points_t: *const f32,
    points_w: *const f32,
}

impl LFAngularPlane {
    unsafe fn as_rust(self: &Self) -> AngularPlane<f32> {
        let s = slice::from_raw_parts(self.points_s, self.num_points).to_owned();
        let t = slice::from_raw_parts(self.points_t, self.num_points).to_owned();
        let w = slice::from_raw_parts(self.points_w, self.num_points).to_owned();
        let basis = match self.basis {
            0 => AngularBasis::Pillbox,
            1 => AngularBasis::Dirac,
            _ => panic!("LFAngularPlane.basis enum had unexpected value"),
        };

        AngularPlane{
            ds: self.ds,
            dt: self.dt,
            basis: basis,
            s: s,
            t: t,
            w: w,
        }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
/// C API image geometry representation
pub struct LFImageGeometry {
    ns: usize,
    nt: usize,

    ds: f32,
    dt: f32,

    offset_s: f32,
    offset_t: f32,
}

impl LFImageGeometry {
    fn as_rust(self: &Self) -> ImageGeometry<f32> {
        ImageGeometry{
            ns: self.ns,
            nt: self.nt,

            ds: self.ds,
            dt: self.dt,

            offset_s: self.offset_s,
            offset_t: self.offset_t,
        }
    }

    /*
    fn from_rust(ig: &ImageGeometry<f32>) -> LFImageGeometry {
        LFImageGeometry {
            ns: ig.ns,
            nt: ig.nt,

            ds: ig.ds,
            dt: ig.dt,

            offset_s: ig.offset_s,
            offset_t: ig.offset_t,
        }
    }
    */
}

#[repr(C)]
#[allow(non_snake_case)]
/// C API light field geometry representation
pub struct LFGeometry {
    geom: LFImageGeometry,
    plane: LFAngularPlane,
    to_plane: LFOpticalX,
}

impl LFGeometry {
    pub unsafe fn as_rust(self: &Self) -> LightFieldGeometry<f32> {
        LightFieldGeometry{
            geom: self.geom.as_rust(),
            plane: self.plane.as_rust(),
            to_plane: self.to_plane.as_rust(),
        }
    }
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

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFTransport_new(source: *const LFGeometry,
                              dest: *const LFGeometry,
                              env: *mut Environment) -> *mut Transport<f32> {
    let src_rust = (*source).as_rust();
    let dst_rust = (*dest).as_rust();
    let env = &mut *env;
    let queue = env.queues[0].clone();
    match Transport::new_simple(src_rust, dst_rust, queue) {
        Ok(xport) => Box::into_raw(Box::new(xport)),
        Err(e) => {
            println!("Error creating Transport object: {:?}", e);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFTransport_del(xport: *mut Transport<f32>) -> () {
    Box::from_raw(xport);
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFTransport_forw_view(xport: *mut Transport<f32>,
                                    src: *const f32,
                                    dst: *mut f32,
                                    angle: usize) -> bool {
    let xport = &mut *xport;
    let src_np = xport.src.geom.dimension();
    let dst_np = xport.dst.geom.dimension();
    let src = slice::from_raw_parts(src, src_np);
    let dst = slice::from_raw_parts_mut(dst, dst_np);
    match xport.forw_host(src, dst, angle) {
        Ok(()) => true,
        Err(e) => {
            println!("Error in LFTransport_forw_view: {:?}", e);
            false
        },
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe fn LFTransport_back_view(xport: *mut Transport<f32>,
                                    dst: *const f32,
                                    src: *mut f32,
                                    angle: usize) -> bool {
    let xport = &mut *xport;
    let src_np = xport.src.geom.dimension();
    let dst_np = xport.dst.geom.dimension();
    let src = slice::from_raw_parts_mut(src, src_np);
    let dst = slice::from_raw_parts(dst, dst_np);
    match xport.back_host(dst, src, angle) {
        Ok(()) => true,
        Err(e) => {
            println!("Error in LFTransport_back_view: {:?}", e);
            false
        },
    }
}

