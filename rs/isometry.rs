extern crate num;
extern crate toml;
extern crate nalgebra;
extern crate byteorder;
use cl_traits::*;
use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use self::nalgebra::{Isometry3, Vector3, Rotation3, Matrix3, BaseFloat, inverse, ApproxEq};
use self::byteorder::*;

/// Isometry for placing objects in space
pub type Isometry<F> = Isometry3<F>;

/// Point in 3d space
pub type Vector<F> = Vector3<F>;

/// Rotation in 3d space
pub type Rotation<F> = Rotation3<F>;

pub fn rotation_x<F: Float + FromPrimitive + BaseFloat>(x_deg: F) -> Rotation<F> {
    let x_rad = F::pi() * x_deg / F::from_f32(180f32).unwrap();

    let m = Matrix3 {
        m11: F::one(),
        m12: F::zero(),
        m13: F::zero(),

        m21: F::zero(),
        m31: F::zero(),

        m22: x_rad.cos(),
        m32: x_rad.sin(),

        m23: -x_rad.sin(),
        m33: x_rad.cos(),
    };
    unsafe { Rotation::new_with_matrix(m) }
}

pub fn rotation_y<F: Float + FromPrimitive + BaseFloat>(y_deg: F) -> Rotation<F> {
    let y_rad = F::pi() * y_deg / F::from_f32(180f32).unwrap();

    let m = Matrix3 {
        m21: F::zero(),
        m22: F::one(),
        m23: F::zero(),

        m12: F::zero(),
        m32: F::zero(),

        m11: y_rad.cos(),
        m31: y_rad.sin(),

        m13: -y_rad.sin(),
        m33: y_rad.cos(),
    };
    unsafe { Rotation::new_with_matrix(m) }
}

pub fn rotation_z<F: Float + FromPrimitive + BaseFloat>(z_deg: F) -> Rotation<F> {
    let z_rad = F::pi() * z_deg / F::from_f32(180f32).unwrap();

    let m = Matrix3 {
        m31: F::zero(),
        m32: F::zero(),
        m33: F::one(),

        m13: F::zero(),
        m23: F::zero(),

        m11: z_rad.cos(),
        m21: z_rad.sin(),

        m12: -z_rad.sin(),
        m22: z_rad.cos(),
    };
    unsafe { Rotation::new_with_matrix(m) }
}

pub fn rotation_from_euler_angles<F: Float + FromPrimitive + BaseFloat>(x_deg: F,
                                                                        y_deg: F,
                                                                        z_deg: F)
                                                                        -> Rotation<F> {
    rotation_z(z_deg) * rotation_y(y_deg) * rotation_x(x_deg)
}

/// Decomposition of a 3D rotation into three shears and a rescaling
///
/// Decomposes a 3D coordinate rotation `R` into the `R = S*Z*X*Y`, where
/// `S` is a scaling (diagonal) transformation, and `X`, `Y` and `Z` are 
/// one-dimensional shears.
#[derive(Clone)]
pub struct ShearDecomposition<F: Float> {
    pub sx: F,
    pub sy: F,
    pub sz: F,

    pub xy: F,
    pub xz: F,

    pub yx: F,
    pub yz: F,

    pub zx: F,
    pub zy: F,
}

impl<F: Float + BaseFloat + ApproxEq<F>> ShearDecomposition<F> {
    pub fn new(rot: &Rotation3<F>) -> Self {
        let from = rot.submatrix();

        let yx_tilde = from.m21;
        let yy_tilde = from.m22;
        let yz_tilde = from.m23;
        let y_tilde = Matrix3 {
            m11: F::one(),
            m12: F::zero(),
            m13: F::zero(),

            m21: yx_tilde,
            m22: yy_tilde,
            m23: yz_tilde,

            m31: F::zero(),
            m32: F::zero(),
            m33: F::one(),
        };

        let mut tmp = *from * inverse(&y_tilde).unwrap();

        let xx_tilde = tmp.m11;
        let xy_tilde = tmp.m12;
        let xz_tilde = tmp.m13;

        let x_tilde = Matrix3 {
            m11: xx_tilde,
            m12: xy_tilde,
            m13: xz_tilde,

            m21: F::zero(),
            m22: F::one(),
            m23: F::zero(),

            m31: F::zero(),
            m32: F::zero(),
            m33: F::one(),
        };

        tmp = tmp * inverse(&x_tilde).unwrap();

        let zx_tilde = tmp.m31;
        let zy_tilde = tmp.m32;
        let zz_tilde = tmp.m33;

        ShearDecomposition {
            sx: xx_tilde,
            sy: yy_tilde,
            sz: zz_tilde,

            xy: xy_tilde * yy_tilde / xx_tilde,
            xz: xz_tilde / xx_tilde,

            yx: yx_tilde / yy_tilde,
            yz: yz_tilde / yy_tilde,

            zx: zx_tilde * xx_tilde / zz_tilde,
            zy: zy_tilde * yy_tilde / zz_tilde,
        }
    }

    pub fn shear_x(self: &Self) -> Matrix3<F> {
        Matrix3 {
            m11: F::one(),
            m12: self.xy,
            m13: self.xz,

            m21: F::zero(),
            m22: F::one(),
            m23: F::zero(),

            m31: F::zero(),
            m32: F::zero(),
            m33: F::one(),
        }
    }

    pub fn shear_y(self: &Self) -> Matrix3<F> {
        Matrix3 {
            m11: F::one(),
            m12: F::zero(),
            m13: F::zero(),

            m21: self.yx,
            m22: F::one(),
            m23: self.yz,

            m31: F::zero(),
            m32: F::zero(),
            m33: F::one(),
        }
    }

    pub fn shear_z(self: &Self) -> Matrix3<F> {
        Matrix3 {
            m11: F::one(),
            m12: F::zero(),
            m13: F::zero(),

            m21: F::zero(),
            m22: F::one(),
            m23: F::zero(),

            m31: self.zx,
            m32: self.zy,
            m33: F::one(),
        }
    }

    pub fn scale(self: &Self) -> Matrix3<F> {
        Matrix3 {
            m11: self.sx,
            m12: F::zero(),
            m13: F::zero(),

            m21: F::zero(),
            m22: self.sy,
            m23: F::zero(),

            m31: F::zero(),
            m32: F::zero(),
            m33: self.sz,
        }
    }
}

impl<F> ClHeader for Isometry<F> {
    fn header() -> &'static str {
        include_str!("../cl/isometry_f32.opencl")
    }
}

impl<F: ToPrimitive> ClBuffer for Isometry<F> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> () {
        // NOTE!
        // According to the OpenCL spec, float3 types (which is how we
        // define Isometry in `isometry_f32.opencl`) are stored as 4-element
        // vectors.  When packing data for OpenCL in this function, we
        // insert those extra spaces.
        //
        // https://www.khronos.org/registry/cl/sdk/1.2/docs/man/xhtml/dataTypes.html

        let rot = &self.rotation.submatrix();
        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m11).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m21).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m31).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(0f32).unwrap();

        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m12).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m22).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m32).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(0f32).unwrap();

        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m13).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m23).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&rot.m33).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(0f32).unwrap();

        let xl = &self.translation;
        buf.write_f32::<LittleEndian>(F::to_f32(&xl.x).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&xl.y).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&xl.z).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(0f32).unwrap();
    }
}

impl<F: Float + FromPrimitive + ToPrimitive + BaseFloat> Serialize for Isometry<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let axes = map.get("axes");
        let origin = map.get("origin");

        match (axes, origin) {
            (Some(&Value::Table(ref axes_tab)),
             Some(&Value::Table(ref origin_tab))) => {
                match (Rotation::from_map(axes_tab), Vector::from_map(origin_tab)) {
                    (Some(axes), Some(origin)) => {
                        Some(Isometry::new_with_rotation_matrix(origin, axes))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("axes".to_string(), Value::Table(self.rotation.into_map()));
        tr.insert("origin".to_string(),
                  Value::Table(self.translation.into_map()));
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
             Some(&Value::Float(z))) => {
                Some(Vector3::new(F::from_f64(x).unwrap(),
                                  F::from_f64(y).unwrap(),
                                  F::from_f64(z).unwrap()))
            }
            _ => None,
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

        let rot_x = map.get("rot_x");
        let rot_y = map.get("rot_y");
        let rot_z = map.get("rot_z");

        match (rot_x, rot_y, rot_z) {
            (Some(&Value::Float(rot_x)),
             Some(&Value::Float(rot_y)),
             Some(&Value::Float(rot_z))) => {
                return Some(rotation_from_euler_angles(F::from_f64(rot_x).unwrap(),
                                                       F::from_f64(rot_y).unwrap(),
                                                       F::from_f64(rot_z).unwrap()))
            }
            _ => {}
        };

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
                let mtx = Matrix3::new(F::from_f64(xx).unwrap(),
                                       F::from_f64(xy).unwrap(),
                                       F::from_f64(xz).unwrap(),
                                       F::from_f64(yx).unwrap(),
                                       F::from_f64(yy).unwrap(),
                                       F::from_f64(yz).unwrap(),
                                       F::from_f64(zx).unwrap(),
                                       F::from_f64(zy).unwrap(),
                                       F::from_f64(zz).unwrap());
                Some(unsafe { Rotation::new_with_matrix(mtx) })
            }
            _ => None,
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
fn test_shear_decomp() {
    let rot: Rotation<f32> = Rotation::new_with_euler_angles(1.0, 2.0, 3.0);
    let rot_mtx = rot.submatrix();
    let decomp = ShearDecomposition::new(&rot);
    let decomp_mtx = decomp.scale() * decomp.shear_z() * decomp.shear_x() * decomp.shear_y();

    assert!((rot_mtx.m11 - decomp_mtx.m11).abs() < 1e-6);
    assert!((rot_mtx.m21 - decomp_mtx.m21).abs() < 1e-6);
    assert!((rot_mtx.m31 - decomp_mtx.m31).abs() < 1e-6);

    assert!((rot_mtx.m12 - decomp_mtx.m12).abs() < 1e-6);
    assert!((rot_mtx.m22 - decomp_mtx.m22).abs() < 1e-6);
    assert!((rot_mtx.m32 - decomp_mtx.m32).abs() < 1e-6);

    assert!((rot_mtx.m13 - decomp_mtx.m13).abs() < 1e-6);
    assert!((rot_mtx.m23 - decomp_mtx.m23).abs() < 1e-6);
    assert!((rot_mtx.m33 - decomp_mtx.m33).abs() < 1e-6);
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
