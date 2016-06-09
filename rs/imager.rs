extern crate nalgebra;
extern crate proust;
extern crate num;
use self::num::{FromPrimitive, Float};
use self::nalgebra::{Rotation3, Vector3};
use self::proust::*;
use geom::*;
use angular_plane::*;

/// Abstract type for a camera at a location that can image an object
pub trait Imager<F, ObjectGeometry, CameraDescription> 
where F: Float + FromPrimitive,
      ObjectGeometry: Geometry<F>,
      Self: Sized {
    fn new(object_geometry: ObjectGeometry,
           camera_description: CameraDescription,
           camera_position: Vector3<F>,
           camera_rotation: Option<Rotation3<F>>,
           na: usize,
           basis: AngularBasis,
           queue: CommandQueue) -> Result<Self, Error>;

    fn na(self: &Self) -> usize;

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

    fn forw_subset<T: Iterator<Item = usize>>(self: &mut Self,
                                              object: &Mem,
                                              view: &mut Mem,
                                              mut angles: T,
                                              wait_for: &[Event]) -> Result<Event, Error> {
        let mut evt = if let Some(ia) = angles.next() {
            try!(self.forw_angle(object, view, ia, wait_for))
        } else {
            return Ok(wait_for[0].clone())
        };

        for ia in angles {
            evt = try!(self.forw_angle(object, view, ia, &[evt]));
        }

        Ok(evt)
    }

    fn back_subset<T: Iterator<Item = usize>>(self: &mut Self,
                                              view: &Mem,
                                              object: &mut Mem,
                                              mut angles: T,
                                              wait_for: &[Event]) -> Result<Event, Error> {
        let mut evt = if let Some(ia) = angles.next() {
            try!(self.back_angle(view, object, ia, wait_for))
        } else {
            return Ok(wait_for[0].clone())
        };

        for ia in angles {
            evt = try!(self.back_angle(view, object, ia, &[evt]));
        }

        Ok(evt)
    }

    fn forw(self: &mut Self,
            object: &Mem,
            view: &mut Mem,
            wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.na();
        self.forw_subset(object, view, 0 .. na, wait_for)
    }

    fn back(self: &mut Self,
            view: &Mem,
            object: &mut Mem,
            wait_for: &[Event]) -> Result<Event, Error> {
        let na = self.na();
        self.back_subset(view, object, 0 .. na, wait_for)
    }
}

