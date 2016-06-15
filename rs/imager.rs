extern crate proust;
extern crate num;
use self::num::{FromPrimitive, Float};
use self::proust::*;
use geom::*;
use detector::*;

/// Abstract type for a camera at a location that can image an object
pub trait Imager<F, ObjectGeometry>
where F: Float + FromPrimitive,
      ObjectGeometry: Geometry<F> {
    /// Number of angles in the imager's angular discretization
    fn na(self: &Self) -> usize;

    /// The imager's detector
    fn detector(self: &Self) -> &Detector<F>;

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

