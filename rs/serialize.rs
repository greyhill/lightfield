extern crate toml;
use self::toml::*;

/// Convert to and from a TOML `Table`
pub trait Serialize where Self: Sized {
    /// Convert from a TOML `Table` if possible
    fn from_map(map: &Table) -> Option<Self>;
    /// Convert from this object into a TOML `Table`
    fn into_map(self: &Self) -> Table;
}

