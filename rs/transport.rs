extern crate num;
extern crate proust;

use light_field_geom::*;
use angular_plane::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::proust::*;
use image_geom::*;
use cl_traits::*;
use optics::*;
use std::mem::{size_of, swap};
use std::cmp::max;

/// Transport between two planes in a light transport stack
pub struct Transport<F: Float> {
    pub src: LightFieldGeometry<F>,
    pub dst: LightFieldGeometry<F>,

    pub overwrite_forw: bool,
    pub overwrite_back: bool,

    pub conservative_forw: bool,
    pub conservative_back: bool,

    pub onto_detector: bool,

    src_s0: usize,
    src_s1: usize,
    src_t0: usize,
    src_t1: usize,

    dst_s0: usize,
    dst_s1: usize,
    dst_t0: usize,
    dst_t1: usize,

    queue: CommandQueue,
    kernel_s: Kernel,
    kernel_t: Kernel,

    src_geom_buf: Mem,
    dst_geom_buf: Mem,
    tmp_buf: Mem,

    src_to_dst: Optics<F>,
    dst_to_src: Optics<F>,
}

impl<F: Float + FromPrimitive + ToPrimitive> Transport<F> {
    pub fn new_simple(src: LightFieldGeometry<F>,
                      dst: LightFieldGeometry<F>,
                      queue: CommandQueue)
                      -> Result<Self, Error> {
        Self::new(src, dst, None, None, true, true, false, false, false, queue)
    }

    pub fn new(src: LightFieldGeometry<F>,
               dst: LightFieldGeometry<F>,
               src_bounds: Option<(usize, usize, usize, usize)>,
               dst_bounds: Option<(usize, usize, usize, usize)>,
               overwrite_forw: bool,
               overwrite_back: bool,
               conservative_forw: bool,
               conservative_back: bool,
               onto_detector: bool,
               queue: CommandQueue)
               -> Result<Self, Error> {
        // collect opencl sources
        let sources = match (&src.plane.basis, &dst.plane.basis) {
            (&AngularBasis::Pillbox, &AngularBasis::Pillbox) => {
                [ImageGeometry::<F>::header(),
                 Optics::<F>::header(),
                 include_str!("../cl/transport_pillbox_f32.opencl")]
            }
            (&AngularBasis::Dirac, &AngularBasis::Dirac) => {
                [ImageGeometry::<F>::header(),
                 Optics::<F>::header(),
                 include_str!("../cl/transport_dirac_f32.opencl")]
            }
            _ => {
                panic!("Cannot transport between light fields with different bases; use a rebin \
                        first");
            }
        };

        // compile opencl code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let unbuilt = try!(Program::new_from_source(context.clone(), &sources));
        let program = try!(unbuilt.build(&[device]));

        // build opencl kernels
        let kernel_s = try!(program.create_kernel("transport_s"));
        let kernel_t = try!(program.create_kernel("transport_t"));

        // use default bounds if none given
        let resolved_src_bounds = if let Some(bounds) = src_bounds {
            bounds
        } else {
            (0, src.geom.ns, 0, src.geom.nt)
        };
        let resolved_dst_bounds = if let Some(bounds) = dst_bounds {
            bounds
        } else {
            (0, dst.geom.ns, 0, dst.geom.nt)
        };

        // create temporary buffer
        let tmp_np = max(resolved_src_bounds.1 - resolved_src_bounds.0,
                         resolved_dst_bounds.1 - resolved_dst_bounds.0) *
                     max(resolved_src_bounds.3 - resolved_src_bounds.2,
                         resolved_dst_bounds.3 - resolved_dst_bounds.2);
        let tmp_buf = try!(queue.create_buffer(size_of::<F>() * tmp_np));

        // create other buffers
        let src_geom_buf = try!(src.geom.as_cl_buffer(&queue));
        let dst_geom_buf = try!(dst.geom.as_cl_buffer(&queue));

        Ok(Transport {
            overwrite_forw: overwrite_forw,
            overwrite_back: overwrite_back,

            conservative_forw: conservative_forw,
            conservative_back: conservative_back,

            onto_detector: onto_detector,

            src_s0: resolved_src_bounds.0,
            src_s1: resolved_src_bounds.1,
            src_t0: resolved_src_bounds.2,
            src_t1: resolved_src_bounds.3,

            dst_s0: resolved_dst_bounds.0,
            dst_s1: resolved_dst_bounds.1,
            dst_t0: resolved_dst_bounds.2,
            dst_t1: resolved_dst_bounds.3,

            queue: queue,
            kernel_s: kernel_s,
            kernel_t: kernel_t,

            tmp_buf: tmp_buf,

            src_geom_buf: src_geom_buf,
            dst_geom_buf: dst_geom_buf,

            src_to_dst: dst.to_plane.invert().compose(&src.to_plane),
            dst_to_src: src.to_plane.invert().compose(&dst.to_plane),

            src: src,
            dst: dst,
        })
    }

    fn bind_common_args(self: &Self, forw: bool, kernel: &mut Kernel) -> Result<(), Error> {
        if forw {
            try!(kernel.bind(0, &self.src_geom_buf));
            try!(kernel.bind(1, &self.dst_geom_buf));

            try!(kernel.bind_scalar(2, &(self.src_s0 as i32)));
            try!(kernel.bind_scalar(3, &(self.src_s1 as i32)));
            try!(kernel.bind_scalar(4, &(self.src_t0 as i32)));
            try!(kernel.bind_scalar(5, &(self.src_t1 as i32)));

            try!(kernel.bind_scalar(6, &(self.dst_s0 as i32)));
            try!(kernel.bind_scalar(7, &(self.dst_s1 as i32)));
            try!(kernel.bind_scalar(8, &(self.dst_t0 as i32)));
            try!(kernel.bind_scalar(9, &(self.dst_t1 as i32)));
        } else {
            try!(kernel.bind(0, &self.dst_geom_buf));
            try!(kernel.bind(1, &self.src_geom_buf));

            try!(kernel.bind_scalar(2, &(self.dst_s0 as i32)));
            try!(kernel.bind_scalar(3, &(self.dst_s1 as i32)));
            try!(kernel.bind_scalar(4, &(self.dst_t0 as i32)));
            try!(kernel.bind_scalar(5, &(self.dst_t1 as i32)));

            try!(kernel.bind_scalar(6, &(self.src_s0 as i32)));
            try!(kernel.bind_scalar(7, &(self.src_s1 as i32)));
            try!(kernel.bind_scalar(8, &(self.src_t0 as i32)));
            try!(kernel.bind_scalar(9, &(self.src_t1 as i32)));
        }

        Ok(())
    }

    #[allow(non_snake_case)] // allow us to break style guide to match docs
    fn transport_dirac_t(self: &mut Self,
                         forw: bool,
                         src: &Mem,
                         ia: usize,
                         wait_for: &[Event])
                         -> Result<Event, Error> {
        let Rqp = if forw {
            &self.src_to_dst
        } else {
            &self.dst_to_src
        };
        let Rp = if forw {
            &self.src.to_plane
        } else {
            &self.dst.to_plane
        };
        let into_geom = if forw {
            &self.dst.geom
        } else {
            &self.src.geom
        };
        let plane = &self.src.plane;

        let t = plane.t[ia];
        let c2 = F::from_f32(2f32).unwrap();

        let alpha = Rqp.tt - Rp.tt * Rqp.tv / Rp.tv;
        let beta = Rqp.t + Rqp.tv * (t - Rp.t) / Rp.tv;
        let mut h = (plane.dt / Rp.tv).abs();
        if !self.onto_detector {
            h = h / self.dst.pixel_volume();
        } else {
            h = h / self.dst.pixel_area() * self.dst.plane.w[ia];
        }

        let mut tau0 = (-into_geom.dt / c2 - beta) / alpha;
        let mut tau1 = (into_geom.dt / c2 - beta) / alpha;
        if tau0 > tau1 {
            swap(&mut tau0, &mut tau1);
        }

        let mut kernel = self.kernel_t.clone();
        try!(self.bind_common_args(forw, &mut kernel));

        try!(kernel.bind_scalar(10, &F::to_f32(&(F::one() / alpha)).unwrap()));
        try!(kernel.bind_scalar(11, &F::to_f32(&tau0).unwrap()));
        try!(kernel.bind_scalar(12, &F::to_f32(&tau1).unwrap()));
        try!(kernel.bind_scalar(13, &F::to_f32(&h).unwrap()));

        try!(kernel.bind(14, src));
        try!(kernel.bind_mut(15, &mut self.tmp_buf));

        let local_size = (32usize, 8usize, 1usize);
        let global_size = if forw {
            (self.src_s1 - self.src_s0, self.dst_t1 - self.dst_t0, 1)
        } else {
            (self.dst_s1 - self.dst_s0, self.src_t1 - self.src_t0, 1)
        };

        self.queue.run_with_events(&mut kernel, local_size, global_size, wait_for)
    }

    #[allow(non_snake_case)] // allow us to break style guide to match docs
    fn transport_dirac_s(self: &mut Self,
                         forw: bool,
                         dst: &mut Mem,
                         ia: usize,
                         wait_for: &[Event])
                         -> Result<Event, Error> {
        let Rqp = if forw {
            &self.src_to_dst
        } else {
            &self.dst_to_src
        };
        let Rp = if forw {
            &self.src.to_plane
        } else {
            &self.dst.to_plane
        };
        let into_geom = if forw {
            &self.dst.geom
        } else {
            &self.src.geom
        };
        let conservative_flag = match (forw, self.conservative_forw, self.conservative_back) {
            (true, true, _) => 1u32,
            (true, false, _) => 0u32,
            (false, _, true) => 1u32,
            (false, _, false) => 0u32,
        };
        let overwrite_flag = match (forw, self.overwrite_forw, self.overwrite_back) {
            (true, true, _) => 1u32,
            (true, false, _) => 0u32,
            (false, _, true) => 1u32,
            (false, _, false) => 0u32,
        };
        let plane = &self.src.plane;

        let s = plane.s[ia];
        let c2 = F::from_f32(2f32).unwrap();

        let alpha = Rqp.ss - Rp.ss * Rqp.su / Rp.su;
        let beta = Rqp.s + Rqp.su * (s - Rp.s) / Rp.su;
        let h = (plane.ds / Rp.su).abs();

        let mut tau0 = (-into_geom.ds / c2 - beta) / alpha;
        let mut tau1 = (into_geom.ds / c2 - beta) / alpha;
        if tau0 > tau1 {
            swap(&mut tau0, &mut tau1);
        }

        let mut kernel = self.kernel_s.clone();
        try!(self.bind_common_args(forw, &mut kernel));

        try!(kernel.bind_scalar(10, &F::to_f32(&(F::one() / alpha)).unwrap()));
        try!(kernel.bind_scalar(11, &F::to_f32(&tau0).unwrap()));
        try!(kernel.bind_scalar(12, &F::to_f32(&tau1).unwrap()));
        try!(kernel.bind_scalar(13, &F::to_f32(&h).unwrap()));

        try!(kernel.bind(14, &self.tmp_buf));
        try!(kernel.bind_mut(15, dst));

        try!(kernel.bind_scalar(16, &conservative_flag));
        try!(kernel.bind_scalar(17, &overwrite_flag));

        let local_size = (32usize, 8usize, 1usize);
        let global_size = if forw {
            (self.dst_t1 - self.dst_t0, self.dst_s1 - self.dst_s0, 1)
        } else {
            (self.src_t1 - self.src_t0, self.src_s1 - self.src_s0, 1)
        };

        self.queue.run_with_events(&mut kernel, local_size, global_size, wait_for)
    }

    fn forw_dirac(self: &mut Self,
                  src: &Mem,
                  dst: &mut Mem,
                  ia: usize,
                  wait_for: &[Event])
                  -> Result<Event, Error> {
        let done_t = try!(self.transport_dirac_t(true, src, ia, wait_for));
        self.transport_dirac_s(true, dst, ia, &[done_t])
    }

    fn back_dirac(self: &mut Self,
                  dst: &Mem,
                  src: &mut Mem,
                  ia: usize,
                  wait_for: &[Event])
                  -> Result<Event, Error> {
        let done_t = try!(self.transport_dirac_t(false, dst, ia, wait_for));
        self.transport_dirac_s(false, src, ia, &[done_t])
    }

    #[allow(non_snake_case)] // allow us to break style guide to match docs
    fn transport_pillbox_t(self: &mut Self,
                           forw: bool,
                           src: &Mem,
                           ia: usize,
                           wait_for: &[Event])
                           -> Result<Event, Error> {
        let Rqp = if forw {
            &self.src_to_dst
        } else {
            &self.dst_to_src
        };
        let Rp = if forw {
            &self.src.to_plane
        } else {
            &self.dst.to_plane
        };
        let into_geom = if forw {
            &self.dst.geom
        } else {
            &self.src.geom
        };
        let plane = &self.src.plane;

        let t = plane.t[ia];
        let c2 = F::from_f32(2f32).unwrap();

        let alpha = Rqp.tt - Rqp.tv * Rp.tt / Rp.tv;
        let beta = Rqp.tv / Rp.tv;
        let gamma = Rqp.t - Rqp.tv * Rp.t / Rp.tv;
        let mut h = (plane.dt / Rp.tv).abs().min((into_geom.dt / Rqp.tv).abs());
        if !self.onto_detector {
            h = h / self.dst.pixel_volume();
        } else {
            h = h / self.dst.pixel_area() * self.dst.plane.w[ia];
        }

        let mut taus = vec![
            (into_geom.dt/c2 - beta*(t + plane.dt/c2) - gamma)/alpha,
            (into_geom.dt/c2 - beta*(t - plane.dt/c2) - gamma)/alpha,
            (-into_geom.dt/c2 - beta*(t + plane.dt/c2) - gamma)/alpha,
            (-into_geom.dt/c2 - beta*(t - plane.dt/c2) - gamma)/alpha,
        ];
        taus.sort_by(|l, r| l.partial_cmp(r).unwrap());

        let mut kernel = self.kernel_t.clone();
        try!(self.bind_common_args(forw, &mut kernel));

        try!(kernel.bind_scalar(10, &F::to_f32(&(F::one() / alpha)).unwrap()));
        try!(kernel.bind_scalar(11, &F::to_f32(&taus[0]).unwrap()));
        try!(kernel.bind_scalar(12, &F::to_f32(&taus[1]).unwrap()));
        try!(kernel.bind_scalar(13, &F::to_f32(&taus[2]).unwrap()));
        try!(kernel.bind_scalar(14, &F::to_f32(&taus[3]).unwrap()));
        try!(kernel.bind_scalar(15, &F::to_f32(&h).unwrap()));

        try!(kernel.bind(16, src));
        try!(kernel.bind_mut(17, &mut self.tmp_buf));

        let local_size = (32usize, 8usize, 1usize);
        let global_size = if forw {
            (self.src_s1 - self.src_s0, self.dst_t1 - self.dst_t0, 1)
        } else {
            (self.dst_s1 - self.dst_s0, self.src_t1 - self.src_t0, 1)
        };

        self.queue.run_with_events(&mut kernel, local_size, global_size, wait_for)
    }

    #[allow(non_snake_case)] // allow us to break style guide to match docs
    fn transport_pillbox_s(self: &mut Self,
                           forw: bool,
                           dst: &mut Mem,
                           ia: usize,
                           wait_for: &[Event])
                           -> Result<Event, Error> {
        let Rqp = if forw {
            &self.src_to_dst
        } else {
            &self.dst_to_src
        };
        let Rp = if forw {
            &self.src.to_plane
        } else {
            &self.dst.to_plane
        };
        let into_geom = if forw {
            &self.dst.geom
        } else {
            &self.src.geom
        };
        let conservative_flag = match (forw, self.conservative_forw, self.conservative_back) {
            (true, true, _) => 1u32,
            (true, false, _) => 0u32,
            (false, _, true) => 1u32,
            (false, _, false) => 0u32,
        };
        let overwrite_flag = match (forw, self.overwrite_forw, self.overwrite_back) {
            (true, true, _) => 1u32,
            (true, false, _) => 0u32,
            (false, _, true) => 1u32,
            (false, _, false) => 0u32,
        };
        let plane = &self.src.plane;

        let s = plane.s[ia];
        let c2 = F::from_f32(2f32).unwrap();

        let alpha = Rqp.ss - Rqp.su * Rp.ss / Rp.su;
        let beta = Rqp.su / Rp.su;
        let gamma = Rqp.s - Rqp.su * Rp.s / Rp.su;
        let h = (plane.ds / Rp.su).abs().min((into_geom.ds / Rqp.su).abs());

        let mut taus = vec![
            (into_geom.ds/c2 - beta*(s + plane.ds/c2) - gamma)/alpha,
            (into_geom.ds/c2 - beta*(s - plane.ds/c2) - gamma)/alpha,
            (-into_geom.ds/c2 - beta*(s + plane.ds/c2) - gamma)/alpha,
            (-into_geom.ds/c2 - beta*(s - plane.ds/c2) - gamma)/alpha,
        ];
        taus.sort_by(|l, r| l.partial_cmp(r).unwrap());

        let mut kernel = self.kernel_s.clone();
        try!(self.bind_common_args(forw, &mut kernel));

        try!(kernel.bind_scalar(10, &F::to_f32(&(F::one() / alpha)).unwrap()));
        try!(kernel.bind_scalar(11, &F::to_f32(&taus[0]).unwrap()));
        try!(kernel.bind_scalar(12, &F::to_f32(&taus[1]).unwrap()));
        try!(kernel.bind_scalar(13, &F::to_f32(&taus[2]).unwrap()));
        try!(kernel.bind_scalar(14, &F::to_f32(&taus[3]).unwrap()));
        try!(kernel.bind_scalar(15, &F::to_f32(&h).unwrap()));

        try!(kernel.bind(16, &self.tmp_buf));
        try!(kernel.bind_mut(17, dst));

        try!(kernel.bind_scalar(18, &conservative_flag));
        try!(kernel.bind_scalar(19, &overwrite_flag));

        let local_size = (32usize, 8usize, 1usize);
        let global_size = if forw {
            (self.dst_t1 - self.dst_t0, self.dst_s1 - self.dst_s0, 1)
        } else {
            (self.src_t1 - self.src_t0, self.src_s1 - self.src_s0, 1)
        };

        self.queue.run_with_events(&mut kernel, local_size, global_size, wait_for)
    }

    fn forw_pillbox(self: &mut Self,
                    src: &Mem,
                    dst: &mut Mem,
                    ia: usize,
                    wait_for: &[Event])
                    -> Result<Event, Error> {
        let done_t = try!(self.transport_pillbox_t(true, src, ia, wait_for));
        self.transport_pillbox_s(true, dst, ia, &[done_t])
    }

    fn back_pillbox(self: &mut Self,
                    dst: &Mem,
                    src: &mut Mem,
                    ia: usize,
                    wait_for: &[Event])
                    -> Result<Event, Error> {
        let done_t = try!(self.transport_pillbox_t(false, dst, ia, wait_for));
        self.transport_pillbox_s(false, src, ia, &[done_t])
    }

    /// Transport from source to destination, overwriting on the destination plane
    pub fn forw(self: &mut Self,
                src: &Mem,
                dst: &mut Mem,
                ia: usize,
                wait_for: &[Event])
                -> Result<Event, Error> {
        match &self.src.plane.basis {
            &AngularBasis::Dirac => self.forw_dirac(src, dst, ia, wait_for),
            &AngularBasis::Pillbox => self.forw_pillbox(src, dst, ia, wait_for),
        }
    }

    /// Transport from destination to source, overwriting on the destination plane
    pub fn back(self: &mut Self,
                dst: &Mem,
                src: &mut Mem,
                ia: usize,
                wait_for: &[Event])
                -> Result<Event, Error> {
        match &self.src.plane.basis {
            &AngularBasis::Dirac => self.back_dirac(dst, src, ia, wait_for),
            &AngularBasis::Pillbox => self.back_pillbox(dst, src, ia, wait_for),
        }
    }
}

#[test]
fn test_transport_dirac() {
    use env::*;
    use lens::*;
    use geom::*;

    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

    let lens = Lens {
        center_s: 1f32,
        center_t: -1.5f32,
        radius_s: 20f32,
        radius_t: 15f32,
        focal_length_s: 30f32,
        focal_length_t: 35f32,
    };
    let plane = lens.as_angular_plane(AngularBasis::Dirac, 20);

    let src_geom = ImageGeometry {
        ns: 100,
        nt: 200,
        ds: 1.0,
        dt: 1.1,
        offset_s: 0.5,
        offset_t: 2.9,
    };
    let dst_geom = ImageGeometry {
        ns: 1024,
        nt: 2048,
        ds: 5e-2,
        dt: 3e-2,
        offset_s: -4.0,
        offset_t: 2.1,
    };

    let dst = LightFieldGeometry {
        geom: dst_geom,
        plane: plane.clone(),
        to_plane: Optics::translation(&40f32),
    };
    let src = LightFieldGeometry {
        geom: src_geom,
        plane: plane.clone(),
        to_plane: lens.optics().then(&Optics::translation(&500f32)).invert(),
    };

    let mut transport = Transport::new_simple(src.clone(), dst.clone(), queue.clone()).unwrap();

    let u = src.geom.rands();
    let v = dst.geom.rands();

    let u_buf = queue.create_buffer_from_slice(&u).unwrap();
    let v_buf = queue.create_buffer_from_slice(&v).unwrap();

    // Project
    let mut proj_u_buf = dst.geom.zeros_buf(&queue).unwrap();
    transport.forw(&u_buf, &mut proj_u_buf, 50, &[]).unwrap().wait().unwrap();
    let mut proj_u = dst.geom.zeros();
    queue.read_buffer(&proj_u_buf, &mut proj_u).unwrap();

    // Backproject
    let mut back_v_buf = src.geom.zeros_buf(&queue).unwrap();
    transport.back(&v_buf, &mut back_v_buf, 50, &[]).unwrap().wait().unwrap();
    let mut back_v = src.geom.zeros();
    queue.read_buffer(&back_v_buf, &mut back_v).unwrap();

    let v1 = proj_u.iter().zip(v.iter()).fold(0f32, |s, (ui, vi)| s + ui * vi);
    let v2 = back_v.iter().zip(u.iter()).fold(0f32, |s, (vi, ui)| s + ui * vi);
    let nrmse = (v1 - v2).abs() / v1.abs().max(v2.abs());

    println!("Adjoint NRMSE for Transport-Dirac: {}", nrmse);
    println!("v'Tu: {}", v1);
    println!("(T'v)'u: {}", v2);
    assert!(nrmse < 1e-4);
}

#[test]
fn test_transport_pillbox() {
    use env::*;
    use lens::*;
    use geom::*;

    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

    let lens = Lens {
        center_s: 1f32,
        center_t: -1.5f32,
        radius_s: 20f32,
        radius_t: 15f32,
        focal_length_s: 30f32,
        focal_length_t: 35f32,
    };
    let plane = lens.as_angular_plane(AngularBasis::Pillbox, 20);

    let src_geom = ImageGeometry {
        ns: 100,
        nt: 200,
        ds: 1.0,
        dt: 2.1,
        offset_s: 0.5,
        offset_t: 0.9,
    };
    let dst_geom = ImageGeometry {
        ns: 1024,
        nt: 2048,
        ds: 2e-2,
        dt: 3e-2,
        offset_s: -4.0,
        offset_t: 2.1,
    };

    let dst = LightFieldGeometry {
        geom: dst_geom,
        plane: plane.clone(),
        to_plane: Optics::translation(&40f32),
    };
    let src = LightFieldGeometry {
        geom: src_geom,
        plane: plane.clone(),
        to_plane: lens.optics().then(&Optics::translation(&500f32)).invert(),
    };

    let mut transport = Transport::new_simple(src.clone(), dst.clone(), queue.clone()).unwrap();

    let u = src.geom.rands();
    let v = dst.geom.rands();

    let u_buf = queue.create_buffer_from_slice(&u).unwrap();
    let v_buf = queue.create_buffer_from_slice(&v).unwrap();

    // Project
    let mut proj_u_buf = dst.geom.zeros_buf(&queue).unwrap();
    transport.forw(&u_buf, &mut proj_u_buf, 50, &[]).unwrap().wait().unwrap();
    let mut proj_u = dst.geom.zeros();
    queue.read_buffer(&proj_u_buf, &mut proj_u).unwrap();

    // Backproject
    let mut back_v_buf = src.geom.zeros_buf(&queue).unwrap();
    transport.back(&v_buf, &mut back_v_buf, 50, &[]).unwrap().wait().unwrap();
    let mut back_v = src.geom.zeros();
    queue.read_buffer(&back_v_buf, &mut back_v).unwrap();

    let v1 = proj_u.iter().zip(v.iter()).fold(0f32, |s, (ui, vi)| s + ui * vi);
    let v2 = back_v.iter().zip(u.iter()).fold(0f32, |s, (vi, ui)| s + ui * vi);
    let nrmse = (v1 - v2).abs() / v1.abs().max(v2.abs());

    println!("Adjoint NRMSE for Transport-Pillbox: {}", nrmse);
    println!("v'Tu: {}", v1);
    println!("(T'v)'u: {}", v2);
    assert!(nrmse < 1e-4);
}
