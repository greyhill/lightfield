use stage::*;
use cpu_context::*;

impl Stage<CpuContext> for [f32] {
    fn forw_angle(self: &Self,
                  context: &mut CpuContext,
                  _: usize, _: usize,
                  input: &[f32],
                  output: &mut [f32]) -> Result<(), ()> {
        let pixels_per_worker = input.len() / context.workers();
        context.scoped(|scope| {
            for ((in_chunk, mask_chunk), out_chunk) in input.chunks(pixels_per_worker)
                .zip(self.chunks(pixels_per_worker))
                    .zip(output.chunks_mut(pixels_per_worker)) {
                scope.execute(move || {
                    for ((in_pixel, mask_pixel), out_pixel) in in_chunk.iter().zip(mask_chunk.iter()).zip(out_chunk.iter_mut()) {
                        *out_pixel = *in_pixel * *mask_pixel;
                    }
                });
            }
        });
        Ok(())
    }

    fn back_angle(self: &Self,
                  context: &mut CpuContext,
                  _: usize, _: usize,
                  input: &[f32],
                  output: &mut [f32]) -> Result<(), ()> {
        self.forw_angle(context, 0, 0, input, output)
    }
}

