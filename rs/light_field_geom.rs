extern crate num;
use self::num::Float;
use image_geom::*;
use angular_plane::*;
use optics::*;

/// One plane in a light transport stack
pub struct LightFieldGeometry<F: Float> {
    pub geom: ImageGeometry<F>,
    pub plane: AngularPlane<F>,
    pub to_plane: Optics<F>,
}

