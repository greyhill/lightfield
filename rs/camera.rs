extern crate num;
extern crate toml;
use self::num::{Float, FromPrimitive, ToPrimitive};
use serialize::*;
use single_lens_camera::*;
use self::toml::*;

#[derive(Debug)]
pub enum CameraConfig<F: Float> {
    SingleLensCamera(SingleLensCamera<F>),
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

