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

#[derive(Clone, Debug)]
pub enum CameraConfig<F: Float> {
    SingleLensCamera(SingleLensCamera<F>),
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
        }
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for CameraConfig<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let slc = SingleLensCamera::<F>::from_map(map);
        match slc {
            Some(slc) => Some(CameraConfig::SingleLensCamera(slc)),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        match self {
            &CameraConfig::SingleLensCamera(ref slc) => slc.into_map(),
        }
    }
}

