extern crate proust;
extern crate num;
extern crate nalgebra;
use self::nalgebra::{BaseFloat, ApproxEq};
use self::num::{FromPrimitive, Float};
use self::proust::*;
use geom::*;
use detector::*;
use imager::*;
use light_volume::*;
use volume_rotation::*;
use vector_math::*;
use angular_plane::*;

pub struct RotatedVolumeImager<F: Float + FromPrimitive> {
    pub rotator: Option<VolumeRotation<F>>,
    pub imager: Box<Imager<F, LightVolume<F>>>,
    vecmath: VectorMath<F>,
    tmp: Option<Mem>,
}

impl<F: Float + BaseFloat + ApproxEq<F> + FromPrimitive> RotatedVolumeImager<F> {
    pub fn new(rotator: Option<VolumeRotation<F>>,
               imager: Box<Imager<F, LightVolume<F>>>,
               queue: CommandQueue)
               -> Result<Self, Error> {
        if let Some(rotator) = rotator {
            let tmp = try!(rotator.dst_geom.zeros_buf(&queue));
            Ok(RotatedVolumeImager {
                rotator: Some(rotator),
                tmp: Some(tmp),
                imager: imager,
                vecmath: try!(VectorMath::new(queue)),
            })
        } else {
            Ok(RotatedVolumeImager {
                rotator: None,
                tmp: None,
                imager: imager,
                vecmath: try!(VectorMath::new(queue)),
            })
        }
    }
}

impl<F: Float + BaseFloat + ApproxEq<F> + FromPrimitive>
Imager<F, LightVolume<F>> for RotatedVolumeImager<F> {
    fn na(self: &Self) -> usize {
        self.imager.na()
    }

    fn detector(self: &Self) -> &Detector<F> {
        self.imager.detector()
    }

    fn geometry(self: &Self) -> &LightVolume<F> {
        self.imager.geometry()
    }

    fn angular_plane(self: &Self) -> &AngularPlane<F> {
        self.imager.angular_plane()
    }

    fn forw_angle(self: &mut Self,
                  object: &Mem,
                  view: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        match (&mut self.rotator, &mut self.tmp) {
            (&mut Some(ref mut rotator), &mut Some(ref mut tmp)) => {
                let evt = try!(rotator.forw(object, tmp, wait_for));
                self.imager.forw_angle(tmp, view, ia, &[evt])
            },
            (&mut None, &mut None) => {
                self.imager.forw_angle(object, view, ia, wait_for)
            },
            _ => panic!("Unexpected state in RotatedVolumeImager")
        }
    }

    fn back_angle(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        match (&mut self.rotator, &mut self.tmp) {
            (&mut Some(ref mut rotator), &mut Some(ref mut tmp)) => {
                let np = self.imager.geometry().dimension();
                let mut evt = try!(self.vecmath.set(np, tmp, F::zero(), wait_for));
                evt = try!(self.imager.back_angle(view, tmp, ia, &[evt]));
                rotator.back(tmp, object, &[evt])
            },
            (&mut None, &mut None) => {
                self.imager.back_angle(view, object, ia, wait_for)
            },
            _ => panic!("Unexpected state in RotatedVolumeImager")
        }
    }

    fn forw_subset(self: &mut Self,
                  object: &Mem,
                  view: &mut Mem,
                  angles: &[usize],
                  wait_for: &[Event]) -> Result<Event, Error> {
        match (&mut self.rotator, &mut self.tmp) {
            (&mut Some(ref mut rotator), &mut Some(ref mut tmp)) => {
                let evt = try!(rotator.forw(object, tmp, wait_for));
                self.imager.forw_subset(tmp, view, angles, &[evt])
            },
            (&mut None, &mut None) => {
                self.imager.forw_subset(object, view, angles, wait_for)
            },
            _ => panic!("Unexpected state in RotatedVolumeImager")
        }
    }

    fn back_subset(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  angles: &[usize],
                  wait_for: &[Event]) -> Result<Event, Error> {
        match (&mut self.rotator, &mut self.tmp) {
            (&mut Some(ref mut rotator), &mut Some(ref mut tmp)) => {
                let np = self.imager.geometry().dimension();
                let mut evt = try!(self.vecmath.set(np, tmp, F::zero(), wait_for));
                evt = try!(self.imager.back_subset(view, tmp, angles, &[evt]));
                rotator.back(tmp, object, &[evt])
            },
            (&mut None, &mut None) => {
                self.imager.back_subset(view, object, angles, wait_for)
            },
            _ => panic!("Unexpected state in RotatedVolumeImager")
        }
    }

}
