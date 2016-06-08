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
}

