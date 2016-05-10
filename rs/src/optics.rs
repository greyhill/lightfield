/// One-dimensional affine geometrical optics
///
/// You can think of an `Optics1d` object as the affine opteration
///
/// ```ignore
/// [ pp pa ] [ p ]       [ cp ]
/// [ ap aa ] [ a ]   +   [ ca ]
/// ```
///
#[derive(Clone, Debug)]
pub struct Optics1d {
    pub pp: f32,
    pub pa: f32,
    pub ap: f32,
    pub aa: f32,
    pub cp: f32,
    pub ca: f32,
}

impl Optics1d {
    pub fn identity() -> Self {
        Optics1d {
            pp: 1f32,
            pa: 0f32,
            ap: 0f32,
            aa: 1f32,
            cp: 0f32,
            ca: 0f32,
        }
    }

    /// Translation
    pub fn translation(d: f32) -> Self {
        Optics1d {
            pp: 1f32,
            pa: d,
            ap: 0f32,
            aa: 1f32,
            cp: 0f32,
            ca: 0f32,
        }
    }

    /// Thin lens geometric refractionion
    pub fn refraction(focal_length: f32, center: f32) -> Self {
        Optics1d {
            pp: 1f32,
            pa: 0f32,
            ap: -1f32 / focal_length,
            aa: 1f32,
            cp: 0f32,
            ca: center / focal_length,
        }
    }

    /// Returns the inverse of the given transformation
    pub fn invert(self: &Self) -> Self {
        unimplemented!()
    }

    /// Perform this optical transformation after the given one
    pub fn compose(self: &Self, rhs: &Self) -> Self {
        unimplemented!()
    }

    /// Perform this optical transformation after the given one
    pub fn after(self: &Self, rhs: &Self) -> Self {
        self.compose(rhs)
    }

    /// Perform this optical transformation before the given one
    pub fn then(self: &Self, lhs: &Self) -> Self {
        lhs.compose(self)
    }
}

/// Separable two-dimensional affine geometrical optics
#[derive(Clone, Debug)]
pub struct Optics2d {
    pub x: Optics1d,
    pub y: Optics1d,
}

impl Optics2d {
    pub fn identity() -> Self {
        Optics2d {
            x: Optics1d::identity(),
            y: Optics1d::identity(),
        }
    }

    /// Translation
    pub fn translation(d: f32) -> Self {
        Optics2d {
            x: Optics1d::translation(d),
            y: Optics1d::translation(d),
        }
    }

    /// Thin lens geometric refractionion
    pub fn refraction(focal_length_x: f32, 
                   center_x: f32,
                   focal_length_y: f32,
                   center_y: f32,) -> Self {
        Optics2d {
            x: Optics1d::refraction(focal_length_x, center_x),
            y: Optics1d::refraction(focal_length_y, center_y),
        }
    }

    /// Returns the inverse of the given transformation
    pub fn invert(self: &Self) -> Self {
        Optics2d {
            x: self.x.invert(),
            y: self.y.invert(),
        }
    }

    /// Perform this optical transformation after the given one
    pub fn compose(self: &Self, rhs: &Self) -> Self {
        Optics2d {
            x: self.x.compose(&rhs.x),
            y: self.y.compose(&rhs.y),
        }
    }

    /// Perform this optical transformation after the given one
    pub fn after(self: &Self, rhs: &Self) -> Self {
        self.compose(rhs)
    }

    /// Perform this optical transformation before the given one
    pub fn then(self: &Self, lhs: &Self) -> Self {
        lhs.compose(self)
    }
}

