extern crate proust;
extern crate num;
use self::num::{FromPrimitive, Float};
use self::proust::*;
use geom::*;
use detector::*;
use angular_plane::*;

/// Abstract type for a camera at a location that can image an object
pub trait Imager<F, ObjectGeometry>
where F: Float + FromPrimitive,
      ObjectGeometry: Geometry<F> {
    /// Number of angles in the imager's angular discretization
    fn na(self: &Self) -> usize;

    /// The imager's detector
    fn detector(self: &Self) -> &Detector<F>;

    /// Object geometry
    fn geometry(self: &Self) -> &ObjectGeometry;

    /// Angular plane for the imager
    fn angular_plane(self: &Self) -> &AngularPlane<F>;

    /// Project a single angle out of the discretization
    fn forw_angle(self: &mut Self,
                  object: &Mem,
                  view: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error>;

    /// Backproject a single angle out of the discretization
    fn back_angle(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error>;

    /// Project an object stored on the host
    ///
    /// This routine is provided for convenience for non-performant code.
    fn forw_host(self: &mut Self, object: &[F], queue: &CommandQueue) -> Result<Vec<F>, Error> {
        let obj = try!(queue.create_buffer_from_slice(object));
        let det_ig = self.detector().image_geometry();
        let mut img = try!(det_ig.zeros_buf(queue));
        let mut img_host = det_ig.zeros();
        try!(try!(self.forw(&obj, &mut img, &[])).wait());
        try!(try!(queue.read_buffer(&img, &mut img_host)).wait());
        Ok(img_host)
    }

    /// Backproject an object stored on the host
    ///
    /// This routine is provided for convenience for non-performant code.
    fn back_host(self: &mut Self, image: &[F], queue: &CommandQueue) -> Result<Vec<F>, Error> {
        let img = try!(queue.create_buffer_from_slice(image));
        let mut obj = try!(self.geometry().zeros_buf(queue));
        let mut obj_host = self.geometry().zeros();
        try!(try!(self.back(&img, &mut obj, &[])).wait());
        try!(try!(queue.read_buffer(&obj, &mut obj_host)).wait());
        Ok(obj_host)
    }

    /// Project all the angles in the discretization
    fn forw(self: &mut Self,
            object: &Mem,
            view: &mut Mem,
            wait_for: &[Event]) -> Result<Event, Error> {
        let angles: Vec<usize> = (0 .. self.na()).collect();
        self.forw_subset(object, view, &angles, wait_for)
    }

    /// Backproject all of the angles in the discretization
    fn back(&mut self,
            view: &Mem,
            object: &mut Mem,
            wait_for: &[Event]) -> Result<Event, Error> {
        let angles: Vec<usize> = (0 .. self.na()).collect();
        self.back_subset(view, object, &angles, wait_for)
    }

    /// Project a subset of the angles in the discretization
    fn forw_subset(self: &mut Self,
                   object: &Mem,
                   view: &mut Mem,
                   angles: &[usize],
                   wait_for: &[Event]) -> Result<Event, Error> {
        let mut evt = try!(self.forw_angle(object, view, 0, wait_for));
        for &ia in angles.iter() {
            evt = try!(self.forw_angle(object, view, ia, &[evt]));
        }
        Ok(evt)
    }

    /// Backproject a subset of the angles in the discretization
    fn back_subset(self: &mut Self,
                   view: &Mem,
                   object: &mut Mem,
                   angles: &[usize],
                   wait_for: &[Event]) -> Result<Event, Error> {
        let mut evt = try!(self.back_angle(view, object, 0, wait_for));
        for &ia in angles.iter() {
            evt = try!(self.back_angle(view, object, ia, &[evt]));
        }
        Ok(evt)
    }
}

