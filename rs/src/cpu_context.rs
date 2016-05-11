use context::*;

pub struct CpuContext {
}

impl Context for Context {
    type Scalar = f32;
    type Vector = [f32];
    type Error = ();
}