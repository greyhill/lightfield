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

    /// Convert integral coordinate to spatial coordinate
    pub fn is2s(self: &Self, is: usize) -> f32 {
        (is as f32 - self.ws())*self.ds
    }

    /// Convert integral coordinate to spatial coordinate
    pub fn it2t(self: &Self, it: usize) -> f32 {
        (it as f32 - self.wt())*self.dt
    }

    /// Convert spatial coordinates to (almost) integral ones
    pub fn s2is(self: &Self, s: f32) -> f32 {
        s/self.ds + self.ws() + 0.5f32
    }

    /// Convert spatial coordinates to (almost) integral ones
    pub fn t2it(self: &Self, t: f32) -> f32 {
        t/self.dt + self.wt() + 0.5f32
    }
}

