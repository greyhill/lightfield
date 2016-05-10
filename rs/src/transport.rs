pub use light_field_geometry::*;

/// Plane-to-plane light field transport
#[derive(Debug)]
pub struct Transport {
    src: LightFieldGeometry,
    dst: LightFieldGeometry,
}

impl Transport {
    pub fn new(src: LightFieldGeometry,
               dst: LightFieldGeometry) -> Self {
        Transport{
            src: src,
            dst: dst,
        }
    }
}

