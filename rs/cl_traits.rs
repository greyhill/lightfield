extern crate proust;
use self::proust::*;

/// Object that has useful headers/defines
pub trait ClHeader {
    /// Returns the constants/defines associated with this object
    fn header() -> &'static str;
}

/// Object that can be represented in OpenCL as a buffer
pub trait ClBuffer {
    /// Serialize this object as a bunch of bytes for OpenCL
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> ();

    /// Calls `as_cl_bytes` and pushes it into an OpenCL buffer
    fn as_cl_buffer(self: &Self, queue: &CommandQueue) -> Result<Mem, Error> {
        let mut b: Vec<u8> = Vec::new();
        self.as_cl_bytes(&mut b);
        queue.create_buffer_from_slice(&b)
    }
}

