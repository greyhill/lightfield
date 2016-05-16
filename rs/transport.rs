extern crate num;
extern crate proust;

use light_field_geom::*;
use angular_plane::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::proust::*;
use image_geom::*;
use cl_traits::*;
use optics::*;
use std::mem::size_of;
use std::cmp::max;

/// Transport between two planes in a light transport stack
pub struct Transport<'src, 'dst, F: 'src + 'dst + Float> {
    pub src: &'src LightFieldGeometry<F>,
    pub dst: &'dst LightFieldGeometry<F>,

    pub src_bounds: (usize, usize, usize, usize),
    pub dst_bounds: (usize, usize, usize, usize),

    queue: CommandQueue,
    forw_kernel_s: Kernel,
    forw_kernel_t: Kernel,
    back_kernel_s: Kernel,
    back_kernel_t: Kernel,

    src_buf: Mem,
    dst_buf: Mem,
    tmp_buf: Mem,
}

impl<'src, 'dst, F: 'src + 'dst + Float + FromPrimitive + ToPrimitive> Transport<'src, 'dst, F> {
    pub fn new(src: &'src LightFieldGeometry<F>, 
               dst: &'dst LightFieldGeometry<F>,
               src_bounds: Option<(usize, usize, usize, usize)>,
               dst_bounds: Option<(usize, usize, usize, usize)>,
               queue: CommandQueue) -> Result<Self, Error> {
        // collect opencl sources
        let sources = match (&src.plane.basis, &dst.plane.basis) {
            (&AngularBasis::Pillbox, &AngularBasis::Pillbox) => {
                [
                    ImageGeometry::<F>::header(),
                    Optics::<F>::header(),
                    include_str!("../cl/transport_pillbox_f32.opencl"),
                ]
            },
            (&AngularBasis::Dirac, &AngularBasis::Dirac) => {
                [
                    ImageGeometry::<F>::header(),
                    Optics::<F>::header(),
                    include_str!("../cl/transport_dirac_f32.opencl"),
                ]
            },
            _ => {
                panic!("Cannot transport between light fields with different bases; use a rebin first");
            },
        };

        // compile opencl code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let unbuilt = try!(Program::new_from_source(context.clone(), &sources));
        let program = try!(unbuilt.build(&[device]));

        // build opencl kernels
        let forw_kernel_s = try!(program.create_kernel("transport_forw_s"));
        let forw_kernel_t = try!(program.create_kernel("transport_forw_t"));
        let back_kernel_s = try!(program.create_kernel("transport_back_s"));
        let back_kernel_t = try!(program.create_kernel("transport_back_t"));

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
                         resolved_dst_bounds.1 - resolved_dst_bounds.0)
            * max(resolved_src_bounds.3 - resolved_src_bounds.2,
                  resolved_dst_bounds.3 - resolved_dst_bounds.2);
        let tmp_buf = try!(queue.create_buffer(size_of::<F>() * tmp_np));

        // create other buffers
        let src_buf = try!(src.geom.as_cl_buffer(&queue));
        let dst_buf = try!(dst.geom.as_cl_buffer(&queue));

        Ok(Transport{
            src: src,
            dst: dst,

            src_bounds: resolved_src_bounds,
            dst_bounds: resolved_dst_bounds,

            queue: queue,
            forw_kernel_s: forw_kernel_s,
            forw_kernel_t: forw_kernel_t,
            back_kernel_s: back_kernel_s,
            back_kernel_t: back_kernel_t,

            tmp_buf: tmp_buf,

            src_buf: src_buf,
            dst_buf: dst_buf,
        })
    }

    pub fn forw(self: &mut Self, 
                src: &Mem, dst: &mut Mem,
                ia: usize, wait_for: &[Event]) -> Result<Event, Error> {
        unimplemented!()
    }

    pub fn back(self: &mut Self,
                dst: &Mem, src: &mut Mem,
                ia: usize, wait_for: &[Event]) -> Result<Event, Error> {
        unimplemented!()
    }
}

