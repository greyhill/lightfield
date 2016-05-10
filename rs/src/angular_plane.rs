use plane_geometry::*;

/// Basis function for angular discretization
#[derive(Clone, Debug)]
pub enum AngularBasis {
    Dirac,
    Pillbox,
}

/// Coordinate used to discretize angles in a system
#[derive(Clone, Debug)]
pub enum AngularCoordinate {
    Space,
    Angle,
}

/// Plane used to parameterize angles in a system
#[derive(Clone, Debug)]
pub struct AngularPlane {
    pub basis: AngularBasis,
    pub coordinate: AngularCoordinate,
    pub geom: PlaneGeometry,
    /// Weights for each angle; e.g., for handling circular (lens) masks
    pub weights: Vec<f32>,
}

