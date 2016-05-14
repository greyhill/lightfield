extern crate num;
extern crate toml;
extern crate nalgebra;
use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use self::nalgebra::{Isometry3, Vector3, Rotation3, Matrix3, BaseFloat};

/// Isometry for placing objects in space
pub type Isometry<F> = Isometry3<F>;

/// Point in 3d space
pub type Vector<F> = Vector3<F>;

/// Rotation in 3d space
pub type Rotation<F> = Rotation3<F>;

impl<F: Float + FromPrimitive + ToPrimitive + BaseFloat> Serialize for Isometry<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let axes = map.get("axes");
        let origin = map.get("origin");

        match (axes, origin) {
            (Some(&Value::Table(ref axes_tab)), 
             Some(&Value::Table(ref origin_tab))) => match (Rotation::from_map(axes_tab), Vector::from_map(origin_tab)) {
                (Some(axes), Some(origin)) => Some(Isometry::new_with_rotation_matrix(origin, axes)),
                _ => None,
            },
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("axes".to_string(), Value::Table(self.rotation.into_map()));
        tr.insert("origin".to_string(), Value::Table(self.translation.into_map()));
        tr
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for Vector<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let x = map.get("x");
        let y = map.get("y");
        let z = map.get("z");

        match (x, y, z) {
            (Some(&Value::Float(x)), 
             Some(&Value::Float(y)),
             Some(&Value::Float(z))) => Some(Vector3::new(
                 F::from_f64(x).unwrap(),
                 F::from_f64(y).unwrap(),
                 F::from_f64(z).unwrap())),
            _ => None
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("x".to_string(), Value::Float(F::to_f64(&self.x).unwrap()));
        tr.insert("y".to_string(), Value::Float(F::to_f64(&self.y).unwrap()));
        tr.insert("z".to_string(), Value::Float(F::to_f64(&self.z).unwrap()));
        tr
    }
}

impl<F: Float + FromPrimitive + ToPrimitive + BaseFloat> Serialize for Rotation<F> {
    fn from_map(map: &Table) -> Option<Self> {
        // TODO: support more ways to read other than just raw matrix entries
        let xx = map.get("xx");
        let yx = map.get("yx");
        let zx = map.get("zx");

        let xy = map.get("xy");
        let yy = map.get("yy");
        let zy = map.get("zy");

        let xz = map.get("xz");
        let yz = map.get("yz");
        let zz = map.get("zz");

        match (xx, yx, zx, xy, yy, zy, xz, yz, zz) {
            (Some(&Value::Float(xx)),
             Some(&Value::Float(yx)),
             Some(&Value::Float(zx)),

             Some(&Value::Float(xy)),
             Some(&Value::Float(yy)),
             Some(&Value::Float(zy)),

             Some(&Value::Float(xz)),
             Some(&Value::Float(yz)),
             Some(&Value::Float(zz))) => {
                let mtx = Matrix3::new(F::from_f64(xx).unwrap(), F::from_f64(xy).unwrap(), F::from_f64(xz).unwrap(),
                                       F::from_f64(yx).unwrap(), F::from_f64(yy).unwrap(), F::from_f64(yz).unwrap(),
                                       F::from_f64(zx).unwrap(), F::from_f64(zy).unwrap(), F::from_f64(zz).unwrap());
                Some(unsafe { Rotation::new_with_matrix(mtx) })
            },
            _ => None
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        let mtx = self.submatrix();

        tr.insert("xx".to_string(), Value::Float(F::to_f64(&mtx.m11).unwrap()));
        tr.insert("yx".to_string(), Value::Float(F::to_f64(&mtx.m21).unwrap()));
        tr.insert("zx".to_string(), Value::Float(F::to_f64(&mtx.m31).unwrap()));

        tr.insert("xy".to_string(), Value::Float(F::to_f64(&mtx.m12).unwrap()));
        tr.insert("yy".to_string(), Value::Float(F::to_f64(&mtx.m22).unwrap()));
        tr.insert("zy".to_string(), Value::Float(F::to_f64(&mtx.m32).unwrap()));

        tr.insert("xz".to_string(), Value::Float(F::to_f64(&mtx.m13).unwrap()));
        tr.insert("yz".to_string(), Value::Float(F::to_f64(&mtx.m23).unwrap()));
        tr.insert("zz".to_string(), Value::Float(F::to_f64(&mtx.m33).unwrap()));

        tr
    }
}

#[test]
fn test_read_vector() {
    let test = r#"
    x = 32.0
    y = 1.2
    z = -15.0
    "#;

    let map = Parser::new(test).parse().unwrap();
    let vec: Vector<f32> = Vector::from_map(&map).unwrap();

    assert_eq!(vec.x, 32.0);
    assert_eq!(vec.y, 1.2);
    assert_eq!(vec.z, -15.0);
}

