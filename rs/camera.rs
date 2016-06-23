extern crate num;
extern crate toml;
extern crate nalgebra;
extern crate proust;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::nalgebra::{Vector3, Rotation3, BaseFloat, ApproxEq};
use serialize::*;
use single_lens_camera::*;
use self::toml::*;
use light_volume::*;
use self::proust::CommandQueue;
use self::proust::Error as PError;
use angular_plane::*;
use imager::*;
use single_lens_imager::*;
use coded_aperture_camera::*;
use coded_aperture_imager::*;
use plenoptic_camera::*;
use plenoptic_imager::*;
use std::path::Path;
use volume_rotation::*;
use rotated_imager::*;

#[derive(Clone, Debug)]
pub enum CameraConfig<F: Float> {
    SingleLensCamera(SingleLensCamera<F>),
    CodedApertureCamera(CodedApertureCamera<F>),
    PlenopticCamera(PlenopticCamera<F>),
}

impl<F: 'static + Float + FromPrimitive + BaseFloat + ApproxEq<F>> CameraConfig<F> {
    pub fn focus_at_distance(self: &mut Self, distance: F) -> () {
        match self {
            &mut CameraConfig::SingleLensCamera(ref mut slc) => slc.focus_at_distance(distance),
            &mut CameraConfig::CodedApertureCamera(ref mut cac) => cac.focus_at_distance(distance),
            &mut CameraConfig::PlenopticCamera(ref mut pc) => pc.focus_at_distance(distance),
        }
    }

    pub fn volume_imager(self: &Self,
                         light_volume: LightVolume<F>,
                         camera_position: Vector3<F>,
                         camera_rotation: Option<Rotation3<F>>,
                         na: usize,
                         basis: AngularBasis,
                         queue: CommandQueue)
                         -> Result<Box<Imager<F, LightVolume<F>>>, PError> {
        let (rotator, geom) = match camera_rotation {
            Some(rot) => {
                let rotator = try!(VolumeRotation::new(&rot, light_volume, queue.clone()));
                let geom = rotator.dst_geom.clone();
                (Some(rotator), geom)
            }
            None => (None, light_volume),
        };

        let internal_imager: Box<Imager<F, LightVolume<F>>> = match self {
            &CameraConfig::SingleLensCamera(ref slc) => {
                Box::new(try!(SingleLensVolumeImager::new(geom,
                                                          slc.clone(),
                                                          camera_position,
                                                          na,
                                                          basis,
                                                          queue.clone())))
            }
            &CameraConfig::CodedApertureCamera(ref cac) => {
                Box::new(try!(CodedApertureVolumeImager::new(geom,
                                                             cac.clone(),
                                                             camera_position,
                                                             na,
                                                             basis,
                                                             queue.clone())))
            }
            &CameraConfig::PlenopticCamera(ref pc) => {
                Box::new(try!(PlenopticVolumeImager::new(geom,
                                                         pc.clone(),
                                                         camera_position,
                                                         na,
                                                         basis,
                                                         queue.clone())))
            }
        };
        Ok(Box::new(try!(RotatedVolumeImager::new(rotator, internal_imager, queue))))
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for CameraConfig<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let slc = SingleLensCamera::<F>::from_map(map);
        let cac = CodedApertureCamera::<F>::from_map(map);
        let pc = PlenopticCamera::<F>::from_map(map);
        match (slc, cac, pc) {
            (Some(slc), _, _) => Some(CameraConfig::SingleLensCamera(slc)),
            (None, Some(cac), _) => Some(CameraConfig::CodedApertureCamera(cac)),
            (None, None, Some(pc)) => Some(CameraConfig::PlenopticCamera(pc)),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        match self {
            &CameraConfig::SingleLensCamera(ref slc) => slc.into_map(),
            &CameraConfig::CodedApertureCamera(ref cac) => cac.into_map(),
            &CameraConfig::PlenopticCamera(ref pc) => pc.into_map(),
        }
    }

    fn load_assets<P: AsRef<Path>>(self: &mut Self, path: P) -> Result<(), ()> {
        match self {
            &mut CameraConfig::SingleLensCamera(ref mut slc) => slc.load_assets(path),
            &mut CameraConfig::CodedApertureCamera(ref mut cac) => cac.load_assets(path),
            &mut CameraConfig::PlenopticCamera(ref mut pc) => pc.load_assets(path),
        }
    }
}
