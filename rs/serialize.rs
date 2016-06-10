extern crate toml;
use self::toml::*;
use std::path::Path;

/// Convert to and from a TOML `Table`
pub trait Serialize where Self: Sized {
    /// Convert from a TOML `Table` if possible
    fn from_map(map: &Table) -> Option<Self>;
    /// Convert from this object into a TOML `Table`
    fn into_map(self: &Self) -> Table;

    /// Load any additional assets from the filesystem
    fn load_assets<P: AsRef<Path>>(self: &mut Self, _: P) -> Result<(), ()> {
        Ok(())
    }
}

