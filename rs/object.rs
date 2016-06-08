extern crate num;
extern crate toml;
use self::num::{Float, FromPrimitive, ToPrimitive};
use serialize::*;
use self::toml::*;
use ellipsoid::*;
use light_volume::*;

#[derive(Debug)]
pub enum ObjectConfig<F: Float> {
    EllipsoidPhantom(Vec<Ellipsoid<F>>),
    LightVolume(LightVolume<F>),
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for ObjectConfig<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let ellipsoids = Vec::<Ellipsoid<_>>::from_map(map);
        let light_volume = LightVolume::from_map(map);
        match (ellipsoids, light_volume) {
            (Some(e), _) => Some(ObjectConfig::EllipsoidPhantom(e)),
            (None, Some(l)) => Some(ObjectConfig::LightVolume(l)),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        match self {
            &ObjectConfig::EllipsoidPhantom(ref v) => v.into_map(),
            &ObjectConfig::LightVolume(ref v) => v.into_map(),
        }
    }
}

