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
use coded_aperture_camera::*;
use mask::*;
use detector::*;
use transport::*;
use geom::*;

/// Implementation of an imager for a volume
pub struct CodedApertureVolumeImager<F: Float + FromPrimitive> {
    volume_xport: VolumeTransport<F>,
    tmp_buf: Mem,
    mask: Mask<F>,
    xport: Transport<F>,
    plane: AngularPlane<F>,
    detector: Detector<F>,
}

impl<F: Float + FromPrimitive> CodedApertureVolumeImager<F> {
    pub fn new(geom: LightVolume<F>,
               camera: CodedApertureCamera<F>,
               position: Vector3<F>,
               na: usize,
               basis: AngularBasis,
               queue: CommandQueue) -> Result<Self, Error> {
        // angular plane on main lens
        let plane = camera.lens.as_angular_plane(basis, na);

        // light field geometry on mask plane
        let mask_lfg = LightFieldGeometry{
            geom: camera.mask_geometry.clone(),
            plane: plane.clone(),
            to_plane: Optics::translation(&camera.distance_lens_mask),
        };

        // light field geometry on detector plane
        let det_lfg = LightFieldGeometry{
            geom: camera.detector.image_geometry(),
            plane: plane.clone(),
            to_plane: Optics::translation(&(camera.distance_lens_mask + camera.distance_detector_mask)),
        };

        // maybe this isn't good design?
        let mask = match camera.mask {
            Some(ref v) => try!(Mask::new(camera.mask_geometry.clone(), v, queue.clone())),
            None => panic!("CodedApertureVolumeImager::new called with unloaded mask"),
        };

        // intermediate buffer 
        let tmp_buf = try!(camera.mask_geometry.zeros_buf(&queue));

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

        // transport from object to mask
        let volume_xport = try!(VolumeTransport::new(frame_geom,
                                                     mask_lfg.clone(),
                                                     optics_object_to_plane,
                                                     true, // overwrite_forw
                                                     false, // overwrite_back
                                                     false, // onto_detector
                                                     queue.clone()));

        let xport = try!(Transport::new(mask_lfg, 
                                        det_lfg,
                                        None, // src bounds
                                        None, // dst bounds
                                        false, // overwrite_forw
                                        false, // overwrite_back
                                        false, // conservative_forw
                                        false, // conservative_back
                                        true, // onto_detector
                                        queue.clone()));

        Ok(CodedApertureVolumeImager{
            volume_xport: volume_xport,
            tmp_buf: tmp_buf,
            mask: mask,
            xport: xport,
            plane: plane,
            detector: camera.detector,
        })
    }
}

impl<F: Float + FromPrimitive> Imager<F, LightVolume<F>> 
for CodedApertureVolumeImager<F> {
    fn na(self: &Self) -> usize {
        self.plane.s.len()
    }

    fn detector(self: &Self) -> &Detector<F> {
        &self.detector
    }

    fn forw_angle(self: &mut Self,
                  object: &Mem,
                  view: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        let mut tmp_copy = self.tmp_buf.clone();
        let mut evt = try!(self.volume_xport.forw(object, &mut tmp_copy, ia, wait_for));
        evt = try!(self.mask.apply_mask(&mut tmp_copy, &[evt]));
        self.xport.forw(&tmp_copy, view, ia, &[evt])
    }

    fn back_angle(self: &mut Self,
                  view: &Mem,
                  object: &mut Mem,
                  ia: usize,
                  wait_for: &[Event]) -> Result<Event, Error> {
        let mut tmp_copy = self.tmp_buf.clone();
        let mut evt = try!(self.xport.back(view, &mut tmp_copy, ia, wait_for));
        evt = try!(self.mask.apply_mask(&mut tmp_copy, &[evt]));
        self.volume_xport.back(&tmp_copy, object, ia, &[evt])
    }
}

