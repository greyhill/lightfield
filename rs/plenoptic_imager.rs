extern crate proust;
extern crate nalgebra;
extern crate num;
use imager::*;
use self::proust::*;
use light_volume::*;
use self::num::{FromPrimitive, Float};
use self::nalgebra::Vector3;
use angular_plane::*;
use volume_transport::*;
use optics::*;
use light_field_geom::*;
use detector::*;
use plenoptic_camera::*;
use lens_array::*;
use geom::*;

pub struct PlenopticVolumeImager<F: Float + FromPrimitive> {
    geom: LightVolume<F>,
    xport: VolumeTransport<F>,
    array: LensArray<F>,
    detector: Detector<F>,
    plane: AngularPlane<F>,
    tmp: Mem,
}

impl<F: Float + FromPrimitive> PlenopticVolumeImager<F> {
    pub fn new(geom: LightVolume<F>,
               camera: PlenopticCamera<F>,
               position: Vector3<F>,
               na: usize,
               basis: AngularBasis,
               queue: CommandQueue) -> Result<Self, Error> {
        // angular plane on main lens
        let plane = camera.lens.as_angular_plane(basis, na);

        // light field geometry on ulens array
        let array_lfg = LightFieldGeometry{
            geom: camera.detector.image_geometry(),
            plane: plane.clone(),
            to_plane: Optics::translation(&camera.distance_lens_array),
        };

        let lenses = match camera.array {
            Some(ref v) => v,
            None => panic!("PlenopticVolumeImager::new called with unloaded lenses"),
        };

        let tmp = try!(camera.detector.image_geometry().zeros_buf(&queue));

        // geometry of the object in the camera's optical frame
        let distance_to_object = -position.z;
        let camera_ox = position.x / geom.dx;
        let camera_oy = position.y / geom.dy;

        // set up geometry of LightVolume in camera's reference frame
        let mut frame_geom = geom.clone();
        frame_geom.offset_x = frame_geom.offset_x + camera_ox;
        frame_geom.offset_y = frame_geom.offset_y + camera_oy;

        let optics_object_to_plane = camera.lens.optics().then(
            &Optics::translation(&distance_to_object)).invert();

        let xport = try!(VolumeTransport::new(frame_geom,
                                              array_lfg.clone(),
                                              optics_object_to_plane,
                                              true, // overwrite_forw
                                              false, // overwrite_back
                                              false, // onto_detector
                                              queue.clone()));

        let array = try!(LensArray::new(array_lfg,
                                        camera.detector.clone(),
                                        camera.distance_detector_array,
                                        lenses,
                                        queue.clone()));

        Ok(PlenopticVolumeImager{
            geom: geom,
            xport: xport,
            array: array,
            detector: camera.detector,
            plane: plane,
            tmp: tmp,
        })
    }
}

impl<F: Float + FromPrimitive> Imager<F, LightVolume<F>>
for PlenopticVolumeImager<F> {
    fn na(self: &Self) -> usize {
        self.plane.s.len()
    }

    fn detector(self: &Self) -> &Detector<F> {
        &self.detector
    }

    fn geometry(self: &Self) -> &LightVolume<F> {
        &self.geom
    }

    fn forw_angle(self: &mut Self,
                  object: &Mem,
                  view: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        let mut tmp_copy = self.tmp.clone();
        let evt = try!(self.xport.forw(object, &mut tmp_copy, ia, wait_for));
        self.array.forw(&tmp_copy, view, ia, &[evt])
    }

    fn back_angle(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        let mut tmp_copy = self.tmp.clone();
        let evt = try!(self.array.back(view, &mut tmp_copy, ia, wait_for));
        self.xport.back(&tmp_copy, object, ia, &[evt])
    }
}

