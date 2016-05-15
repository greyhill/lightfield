/// Object that has useful headers/defines
pub trait ClHeader {
    /// Returns the constants/defines associated with this object
    fn header<S: AsRef<str>>(self: &Self, name: S) -> String;
}

