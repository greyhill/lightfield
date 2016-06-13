extern crate num;
extern crate proust;
use self::num::{FromPrimitive, ToPrimitive, Float};
use image_geom::*;
use self::proust::*;
use cl_traits::*;

/// Spatial mask operation
pub struct Mask<F: Float> {
    geom: ImageGeometry<F>,
    geom_buffer: Mem,
    apply_mask: Kernel,
    apply_mask_to: Kernel,
    mask: Mem,
    queue: CommandQueue,
}

impl<F: Float + FromPrimitive + ToPrimitive> Mask<F> {
    pub fn new(geometry: ImageGeometry<F>,
               mask: &[F],
               queue: CommandQueue) -> Result<Self, Error> {
        let context = try!(queue.context());
        let device = try!(queue.device());
        let source = &[ImageGeometry::<F>::header()];
        let unbuilt = try!(Program::new_from_source(context, source));
        let built = try!(unbuilt.build(&[device]));
        let apply_mask = try!(built.create_kernel("apply_mask"));
        let apply_mask_to = try!(built.create_kernel("apply_mask_to"));
        let mask_buf = try!(queue.create_buffer_from_slice(mask));
        let geom_buffer = try!(geometry.as_cl_buffer(&queue));

        Ok(Mask{
            geom: geometry,
            geom_buffer: geom_buffer,
            apply_mask: apply_mask,
            apply_mask_to: apply_mask_to,
            mask: mask_buf,
            queue: queue,
        })
    }

    /// Apply mask in place
    pub fn apply_mask(self: &mut Self,
                      img: &mut Mem,
                      wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.apply_mask.bind(0, &self.geom_buffer));
        try!(self.apply_mask.bind(1, &self.mask));
        try!(self.apply_mask.bind_mut(2, img));
        
        let local_size = (32, 8, 1);
        let global_size = (self.geom.ns, self.geom.nt, 1);

        self.queue.run_with_events(&mut self.apply_mask,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    /// Apply mask out-of-place
    pub fn apply_mask_to(self: &mut Self,
                      img: &Mem,
                      out: &mut Mem,
                      wait_for: &[Event]) -> Result<Event, Error> {
        try!(self.apply_mask_to.bind(0, &self.geom_buffer));
        try!(self.apply_mask_to.bind(1, &self.mask));
        try!(self.apply_mask_to.bind(2, img));
        try!(self.apply_mask_to.bind_mut(3, out));
        
        let local_size = (32, 8, 1);
        let global_size = (self.geom.ns, self.geom.nt, 1);

        self.queue.run_with_events(&mut self.apply_mask_to,
                                   local_size,
                                   global_size,
                                   wait_for)
    }
}

