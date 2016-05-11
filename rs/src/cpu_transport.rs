use light_field_geometry::*;
use stage::*;

pub struct CpuTransport {
    input: LightFieldGeometry,
    output: LightFieldGeometry,
}

impl CpuTransport {
    pub fn new(input_geometry: LightFieldGeometry,
               output_geometry: LightFieldGeometry) -> Self {
        CpuTransport{
            input: input_geometry,
            output: output_geometry,
        }
    }
}

