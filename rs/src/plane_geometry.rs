/// A 2d plane with a regular rectangular discretization
#[derive(Clone, Debug)]
pub struct PlaneGeometry {
    pub ns: usize,
    pub nt: usize,
    pub ds: f32,
    pub dt: f32,
    pub offset_s: f32,
    pub offset_t: f32,
}

impl PlaneGeometry {
    /// Center of the plane in unitless coordinates
    pub fn ws(self: &Self) -> f32 {
        (self.ns as f32 - 1f32)/2f32 + self.offset_s
    }

    /// Center of the plane in unitless coordinates
    pub fn wt(self: &Self) -> f32 {
        (self.nt as f32 - 1f32)/2f32 + self.offset_t
    }
}

