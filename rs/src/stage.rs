use context::*;

/// Abstraction for a type that processes light fields
pub trait Stage<C: Context> {
    fn forw_angle(self: &Self, 
                  context: &mut C,
                  iu: usize, iv: usize,
                  input: &C::Vector,
                  output: &mut C::Vector) -> Result<(), C::Error>;

    fn back_angle(self: &Self,
                  context: &mut C,
                  iu: usize, iv: usize,
                  input: &C::Vector,
                  output: &mut C::Vector) -> Result<(), C::Error>;
}

