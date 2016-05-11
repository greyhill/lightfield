extern crate scoped_pool;
use self::scoped_pool::Pool;

use context::*;

/// Context for multi-threaded computation
pub type CpuContext = Pool;

impl Context for CpuContext {
    type Scalar = f32;
    type Vector = [f32];
    type Error = ();
}
