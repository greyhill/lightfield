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
use single_lens_camera::*;
use detector::*;

/// Implementation of an imager for a volume
pub struct SingleLensVolumeImager<F: Float + FromPrimitive> {
    geom: LightVolume<F>,
    xport: VolumeTransport<F>,
    plane: AngularPlane<F>,
    detector: Detector<F>,
}

impl<F: Float + FromPrimitive> SingleLensVolumeImager<F> {
    pub fn new(geom: LightVolume<F>,
               camera: SingleLensCamera<F>,
               position: Vector3<F>,
               na: usize,
               basis: AngularBasis,
               queue: CommandQueue) -> Result<Self, Error> {
        // angular plane on main lens
        let plane = camera.lens.as_angular_plane(basis, na);

        // light field geometry on detector
        let detector_lfg = LightFieldGeometry{
            geom: camera.detector.image_geometry(),
            plane: plane.clone(),
            to_plane: Optics::translation(&camera.distance_detector_lens),
        };

        // geometry of the object in the camera's optical frame
        let distance_to_object = -position.z;
        let camera_ox = position.x / geom.dx;
        let camera_oy = position.y / geom.dy;

        let mut frame_geom = geom.clone();
        frame_geom.offset_x = frame_geom.offset_x + camera_ox;
        frame_geom.offset_y = frame_geom.offset_y + camera_oy;

        let optics_object_to_plane = camera.lens.optics().then(
            &Optics::translation(&distance_to_object)).invert();

        // transport from object to detector
        let xport = try!(VolumeTransport::new(frame_geom,
                                              detector_lfg,
                                              optics_object_to_plane,
                                              false, // overwrite_forw
                                              false, // overwrite_back
                                              true, // onto_detector
                                              queue.clone()));

        Ok(SingleLensVolumeImager{
            geom: geom,
            xport: xport,
            plane: plane,
            detector: camera.detector,
        })
    }
}


impl<F: Float + FromPrimitive> Imager<F, LightVolume<F>>
for SingleLensVolumeImager<F> {
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
        self.xport.forw(object, view, ia, wait_for)
    }

    fn back_angle(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        self.xport.back(view, object, ia, wait_for)
    }
}

