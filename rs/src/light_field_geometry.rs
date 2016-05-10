use angular_plane::*;
use plane_geometry::*;
use optics::*;
use float_util::*;

/// Geometry for a light field
///
/// A light field is specified by its own spatial discretization, the
/// optical transformation (an `Optics2d` object) to the angular plane,
/// and the geometry of the angular plane.
#[derive(Clone, Debug)]
pub struct LightFieldGeometry {
    pub geom: PlaneGeometry,
    pub optics_to_plane: Optics2d,
    pub angular_plane: AngularPlane,
}

impl LightFieldGeometry {
    /// Volume of a pixel on the light field.
    ///
    /// This is the diagonal term (denominator) in light field and is computed
    /// from the squared norm of the light field pixel:
    ///
    /// ```ignore
    /// || b((s - s_i)/ds) b((T(s,u) - u_k)/du) ||_2^2 * || b((t - t_i)/dt) b((T(t,v) - v_k)/dv) ||_2^2
    /// ```
    ///
    /// where `T(s,u)` and `T(t,v)` are the coordinate transformations from this plane to the
    /// optical plane.  We use this to to get correct units in light field transport and when
    /// accumulating onto a CCD.
    pub fn pixel_volume(self: &Self) -> f32 {
        match (&self.angular_plane.basis, &self.angular_plane.coordinate) {
            (&AngularBasis::Dirac, &AngularCoordinate::Space) => {
                let vs = (self.angular_plane.geom.ds/self.optics_to_plane.x.pa).abs() * self.geom.ds;
                let vt = (self.angular_plane.geom.dt/self.optics_to_plane.y.pa).abs() * self.geom.dt;
                vs * vt
            },
            (&AngularBasis::Dirac, &AngularCoordinate::Angle) => {
                let vs = (self.angular_plane.geom.ds/self.optics_to_plane.x.aa).abs() * self.geom.ds;
                let vt = (self.angular_plane.geom.dt/self.optics_to_plane.y.aa).abs() * self.geom.dt;
                vs * vt
            },
            (&AngularBasis::Pillbox, &AngularCoordinate::Space) => {
                let sm = fmax(self.angular_plane.geom.ds / 2f32 / self.optics_to_plane.x.pa.abs(),
                              self.geom.ds * self.optics_to_plane.x.pp.abs() 
                                 / 2f32 / self.optics_to_plane.x.pa.abs());
                let sh = fmin(self.geom.ds, self.angular_plane.geom.ds / self.optics_to_plane.x.pp.abs());

                let tm = fmax(self.angular_plane.geom.dt / 2f32 / self.optics_to_plane.y.pa.abs(),
                              self.geom.dt * self.optics_to_plane.y.pp.abs() 
                                 / 2f32 / self.optics_to_plane.y.pa.abs());
                let th = fmin(self.geom.dt, self.angular_plane.geom.dt / self.optics_to_plane.y.pp.abs());

                4f32 * tm * sm * sh * th
            },
            (&AngularBasis::Pillbox, &AngularCoordinate::Angle) => {
                let sm = fmax(self.angular_plane.geom.ds / 2f32 / self.optics_to_plane.x.aa.abs(),
                              self.geom.ds * self.optics_to_plane.x.ap.abs() 
                                 / 2f32 / self.optics_to_plane.x.aa.abs());
                let sh = fmin(self.geom.ds, self.angular_plane.geom.ds / self.optics_to_plane.x.ap.abs());

                let tm = fmax(self.angular_plane.geom.dt / 2f32 / self.optics_to_plane.y.aa.abs(),
                              self.geom.dt * self.optics_to_plane.y.ap.abs() 
                                 / 2f32 / self.optics_to_plane.y.aa.abs());
                let th = fmin(self.geom.dt, self.angular_plane.geom.dt / self.optics_to_plane.y.ap.abs());

                4f32 * tm * sm * sh * th
            },
        }
    }
}

