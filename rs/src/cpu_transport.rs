use light_field_geometry::*;
use stage::*;
use cpu_context::*;

/// Multithreaded plane-to-plane transport
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

impl Stage<CpuContext> for CpuTransport {
    fn forw_angle(self: &Self,
                  context: &mut CpuContext,
                  iu: usize, iv: usize,
                  input: &[f32],
                  output: &mut [f32]) -> Result<(), ()> {
        unimplemented!()
    }

    fn back_angle(self: &Self,
                  context: &mut CpuContext,
                  iu: usize, iv: usize,
                  input: &[f32],
                  output: &mut [f32]) -> Result<(), ()> {
        unimplemented!()
    }
}

