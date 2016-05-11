/// Abstraction for a computational context
pub trait Context {
    type Scalar;
    type Vector;
    type Error;
}
