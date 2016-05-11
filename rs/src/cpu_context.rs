use context::*;

pub struct CpuContext {
    num_threads: usize,
}

impl Context for CpuContext {
    type Scalar = f32;
    type Vector = [f32];
    type Error = ();
}
