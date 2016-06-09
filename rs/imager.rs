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
    fn na(self: &Self) -> usize;

    fn detector(self: &Self) -> &Detector<F>;

    fn forw_angle(self: &mut Self,
                  object: &Mem,
                  view: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error>;

    fn back_angle(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error>;

    fn forw(self: &mut Self,
            object: &Mem,
            view: &mut Mem,
            wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.na();
        let mut evt = try!(self.forw_angle(object, view, 0, wait_for));
        for ia in 1 .. na {
            evt = try!(self.forw_angle(object, view, ia, &[evt]));
        }
        Ok(evt)
    }

    fn back(&mut self,
            view: &Mem,
            object: &mut Mem,
            wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.na();
        let mut evt = try!(self.back_angle(view, object, 0, wait_for));
        for ia in 1 .. na {
            evt = try!(self.back_angle(view, object, ia, &[evt]));
        }
        Ok(evt)
    }
}

