extern crate num;
use self::num::{Float, FromPrimitive};
use image_geom::*;
use angular_plane::*;
use optics::*;

/// One plane in a light transport stack
pub struct LightFieldGeometry<F: Float> {
    pub geom: ImageGeometry<F>,
    pub plane: AngularPlane<F>,
    pub to_plane: Optics<F>,
}

impl<F: Float + FromPrimitive> LightFieldGeometry<F> {
    /// Returns the 4d "squared volume" of a pixel on the light field
    ///
    /// This expression comes from integrating the spatial and angular basis
    /// functions over their domain; see Table 1 in the PDF documentation.
    pub fn pixel_volume(self: &Self) -> F {
        match &self.plane.basis {
            &AngularBasis::Dirac => {
                let vs = (self.plane.ds / self.to_plane.su).abs() * self.geom.ds;
                let vt = (self.plane.dt / self.to_plane.tv).abs() * self.geom.dt;
                vs * vt
            },
            &AngularBasis::Pillbox => {
                let c2 = F::from_f32(2f32).unwrap();

                let s_m_a = self.plane.ds / c2 / self.to_plane.su.abs();
                let s_m_b = self.geom.ds / c2 * (self.to_plane.ss / self.to_plane.su).abs();
                let s_m = s_m_a.max(s_m_b);
                let s_h = self.geom.ds.min(self.plane.ds / self.to_plane.ss.abs());

                let t_m_a = self.plane.dt / c2 / self.to_plane.tv.abs();
                let t_m_b = self.geom.dt / c2 * (self.to_plane.tt / self.to_plane.tv).abs();
                let t_m = t_m_a.max(t_m_b);
                let t_h = self.geom.dt.min(self.plane.dt / self.to_plane.tt.abs());

                c2 * c2 * s_m * s_h * t_m * t_h
            },
        }
    }
}

