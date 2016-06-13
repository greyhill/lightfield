extern crate num;
extern crate toml;
extern crate nalgebra;
extern crate proust;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::nalgebra::{Vector3, Rotation3};
use serialize::*;
use single_lens_camera::*;
use self::toml::*;
use light_volume::*;
use self::proust::{CommandQueue};
use self::proust::Error as PError;
use angular_plane::*;
use imager::*;
use single_lens_imager::*;
use coded_aperture_camera::*;
use coded_aperture_imager::*;
use plenoptic_camera::*;
use plenoptic_imager::*;
use std::path::Path;

#[derive(Clone, Debug)]
pub enum CameraConfig<F: Float> {
    SingleLensCamera(SingleLensCamera<F>),
    CodedApertureCamera(CodedApertureCamera<F>),
    PlenopticCamera(PlenopticCamera<F>),
}

impl<F: 'static + Float + FromPrimitive> CameraConfig<F> {
    pub fn volume_imager(self: &Self,
                         light_volume: LightVolume<F>,
                         camera_position: Vector3<F>,
                         camera_rotation: Option<Rotation3<F>>,
                         na: usize,
                         basis: AngularBasis,
                         queue: CommandQueue) -> Result<Box<Imager<F, LightVolume<F>>>, PError> {
        match self {
            &CameraConfig::SingleLensCamera(ref slc) => Ok(Box::new(try!(SingleLensVolumeImager::new(
                            light_volume,
                            slc.clone(),
                            camera_position,
                            camera_rotation,
                            na,
                            basis,
                            queue,
            )))),
            &CameraConfig::CodedApertureCamera(ref cac) => Ok(Box::new(try!(CodedApertureVolumeImager::new(
                            light_volume,
                            cac.clone(),
                            camera_position,
                            camera_rotation,
                            na,
                            basis,
                            queue,
            )))),
            &CameraConfig::PlenopticCamera(ref pc) => Ok(Box::new(try!(PlenopticVolumeImager::new(
                            light_volume,
                            pc.clone(),
                            camera_position,
                            camera_rotation,
                            na,
                            basis,
                            queue,
            )))),
        }
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

