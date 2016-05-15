extern crate num;
extern crate toml;
use serialize::*;
use optics::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use image_geom::*;
use occluder::*;
use bounding_geometry::*;

/// Ideal thin lens
#[derive(Clone, Debug)]
pub struct Lens<F: Float> {
    pub center_s: F,
    pub center_t: F,
    pub radius_s: F,
    pub radius_t: F,
    pub focal_length_s: F,
    pub focal_length_t: F,
}

impl<F: Float + FromPrimitive> Lens<F> {
    /// Returns the optical transformation from this lens
    pub fn optics(self: &Self) -> Optics<F> {
        Optics::anisotropic_lens(&self.center_s, &self.center_t,
                                 &self.focal_length_s, &self.focal_length_t)
    }
}

impl<F: Float + FromPrimitive> BoundingGeometry<F> for Lens<F> {
    fn bounding_geometry(self: &Self, ns: usize, nt: usize) -> ImageGeometry<F> {
        let ds = F::from_f32(2f32).unwrap() * self.radius_s / F::from_usize(ns).unwrap();
        let dt = F::from_f32(2f32).unwrap() * self.radius_t / F::from_usize(nt).unwrap();
        let offset_s = -self.center_s / ds;
        let offset_t = -self.center_t / dt;
        ImageGeometry{
            ns: ns,
            nt: nt,
            ds: ds,
            dt: dt,
            offset_s: offset_s,
            offset_t: offset_t,
        }
    }
}

impl<F: Float + FromPrimitive> Occluder<F> for Lens<F> {
    fn occludes(self: &Self, s: F, t: F) -> bool {
        let ds = ((s - self.center_s) / self.radius_s).powi(2);
        let dt = ((t - self.center_t) / self.radius_t).powi(2);
        ds + dt >= F::one()
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for Lens<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let center_s = map.get("center_s");
        let center_t = map.get("center_t");
        let radius_s = map.get("radius_s");
        let radius_t = map.get("radius_t");
        let focal_length_s = map.get("focal_length_s");
        let focal_length_t = map.get("focal_length_t");

        match (center_s, center_t, radius_s, radius_t, focal_length_s, focal_length_t) {
            (Some(&Value::Float(center_s)),
             Some(&Value::Float(center_t)),
             Some(&Value::Float(radius_s)),
             Some(&Value::Float(radius_t)),
             Some(&Value::Float(focal_length_s)),
             Some(&Value::Float(focal_length_t))) => Some(Lens{
                center_s: F::from_f64(center_s).unwrap(),
                center_t: F::from_f64(center_t).unwrap(),
                radius_s: F::from_f64(radius_s).unwrap(),
                radius_t: F::from_f64(radius_t).unwrap(),
                focal_length_s: F::from_f64(focal_length_s).unwrap(),
                focal_length_t: F::from_f64(focal_length_t).unwrap(),
            }),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("center_s".to_string(), Value::Float(F::to_f64(&self.center_s).unwrap()));
        tr.insert("center_t".to_string(), Value::Float(F::to_f64(&self.center_t).unwrap()));
        tr.insert("radius_s".to_string(), Value::Float(F::to_f64(&self.radius_s).unwrap()));
        tr.insert("radius_t".to_string(), Value::Float(F::to_f64(&self.radius_t).unwrap()));
        tr.insert("focal_length_s".to_string(), Value::Float(F::to_f64(&self.focal_length_s).unwrap()));
        tr.insert("focal_length_t".to_string(), Value::Float(F::to_f64(&self.focal_length_t).unwrap()));
        tr
    }
}

#[test]
fn test_read_lens() {
    let test = r#"
        center_s = 3.0
        center_t = -5.2
        radius_s = 4.0
        radius_t = 5.2
        focal_length_s = 12.0
        focal_length_t = 24.0
    "#;

    let map = Parser::new(test).parse().unwrap();
    let lens: Lens<f64> = Lens::from_map(&map).unwrap();

    assert_eq!(lens.center_s, 3.0);
    assert_eq!(lens.center_t, -5.2);
    assert_eq!(lens.radius_s, 4.0);
    assert_eq!(lens.radius_t, 5.2);
    assert_eq!(lens.focal_length_s, 12.0);
    assert_eq!(lens.focal_length_t, 24.0);
}
