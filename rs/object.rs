extern crate num;
extern crate toml;
use self::num::{Float, FromPrimitive, ToPrimitive};
use serialize::*;
use self::toml::*;
use light_volume::*;

#[derive(Debug)]
pub enum ObjectConfig<F: Float> {
    LightVolume(LightVolume<F>),
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for ObjectConfig<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let light_volume = LightVolume::from_map(map);
        match light_volume {
            Some(l) => Some(ObjectConfig::LightVolume(l)),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        match self {
            &ObjectConfig::LightVolume(ref v) => v.into_map(),
        }
    }
}

