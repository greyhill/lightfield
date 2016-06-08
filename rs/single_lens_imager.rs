extern crate proust;
extern crate nalgebra;
extern crate num;
use imager::*;
use self::proust::*;
use light_volume::*;
use self::num::{FromPrimitive, Float};
use self::nalgebra::{Rotation3, Vector3};
use single_lens_camera::*;
use angular_plane::*;
use volume_transport::*;
use optics::*;
use light_field_geom::*;

/// Implementation of an imager for a volume
pub struct SingleLensVolumeImager<F: Float + FromPrimitive> {
    xport: VolumeTransport<F>,
    queue: CommandQueue,
}

impl<F: Float + FromPrimitive> Imager<F, LightVolume<F>, SingleLensCamera<F>>
for SingleLensVolumeImager<F> {
    fn new(geom: LightVolume<F>,
           camera: SingleLensCamera<F>,
           position: Vector3<F>,
           rotation: Option<Rotation3<F>>,
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

        // TODO
        assert!(rotation.is_none());

        // geometry of the object in the camera's optical frame
        // TODO: this will require some rejiggering when I add rotations
        let distance_to_object = position.z;
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
            xport: xport,
            queue: queue,
        })
    }

    fn forw_angle(self: &mut Self,
                  object: &Mem,
                  view: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        // TODO -- will need an extra step with non-zero rotation
        self.xport.forw(object, view, ia, wait_for)
    }

    fn back_angle(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        self.xport.back(view, object, ia, wait_for)
        // TODO -- will need extra step with non-zero rotation
    }
}
