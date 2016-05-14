extern crate nalgebra;
extern crate num;

use self::num::Float;
use self::nalgebra::Isometry3;

pub type Isometry<F: Float> = Isometry3<F>;

