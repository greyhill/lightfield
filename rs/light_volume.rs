extern crate num;
extern crate toml;
extern crate byteorder;
extern crate avsfld;
use serialize::*;
use cl_traits::*;
use geom::*;
use self::toml::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::byteorder::*;
use image_geom::*;
use optics::*;
use light_field_geom::*;
use angular_plane::*;
use std::path::Path;
use std::fs::File;

/// Volume of lambertian light emitters
#[derive(Clone, Debug)]
pub struct LightVolume<F: Float> {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub dx: F,
    pub dy: F,
    pub dz: F,
    pub offset_x: F,
    pub offset_y: F,
    pub offset_z: F,
}

impl<F: Float + FromPrimitive> LightVolume<F> {
    pub fn wx(self: &Self) -> F {
        (F::from_usize(self.nx).unwrap() - F::one())/F::from_f32(2f32).unwrap() + self.offset_x
    }

    pub fn wy(self: &Self) -> F {
        (F::from_usize(self.ny).unwrap() - F::one())/F::from_f32(2f32).unwrap() + self.offset_y
    }

    pub fn wz(self: &Self) -> F {
        (F::from_usize(self.nz).unwrap() - F::one())/F::from_f32(2f32).unwrap() + self.offset_z
    }

    /// Returns the image geometry for this slice
    pub fn transaxial_image_geometry(self: &Self) -> ImageGeometry<F> {
        ImageGeometry{
            ns: self.nx,
            nt: self.ny,
            ds: self.dx,
            dt: self.dy,
            offset_s: self.offset_x,
            offset_t: self.offset_y,
        }
    }

    /// Returns the optical transform from a slice to the `z=0` plane in this volume
    pub fn optics_to_z0(self: &Self, iz: usize) -> Optics<F> {
        let z = (F::from_usize(iz).unwrap() - self.wz())*self.dz;
        Optics::translation(&-z)
    }

    /// Returns a `LightFieldGeometry` for a slice
    ///
    /// `iz` is the slice number.  Angles are defined on `plane`, and the
    /// optical transformation from the `z=0` plane of this volume to the 
    /// angular plane is given by `to_plane_from_z0`.
    pub fn slice_light_field_geometry(self: &Self, 
                                      iz: usize,
                                      plane: AngularPlane<F>,
                                      to_plane_from_z0: Optics<F>) -> LightFieldGeometry<F> {
        LightFieldGeometry{
            geom: self.transaxial_image_geometry(),
            plane: plane,
            to_plane: to_plane_from_z0.compose(&self.optics_to_z0(iz)),
        }
    }

}

impl<F: Float + FromPrimitive> Geometry<F> for LightVolume<F> {
    fn shape(self: &Self) -> Vec<usize> {
        vec![self.nx, self.ny, self.nz]
    }

    fn save<P: AsRef<Path>>(self: &Self, buf: &[F], path: P) -> Result<(), ()> {
        let mut file = if let Ok(f) = File::create(path) {
            f
        } else {
            return Err(());
        };
        let sizes = [self.nx, self.ny, self.nz];
        match self::avsfld::AVSFile::write(&mut file, &sizes, buf) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn load<P: AsRef<Path>>(self: &Self, path: P) -> Result<Vec<F>, ()> {
        let mut file = if let Ok(f) = self::avsfld::AVSFile::open(&path) {
            f
        } else {
            return Err(())
        };
        match file.read() {
            Ok(tr) => Ok(tr),
            Err(_) => Err(()),
        }
    }
}

impl<F: Float> ClHeader for LightVolume<F> {
    fn header() -> &'static str {
        include_str!("../cl/light_volume_f32.opencl")
    }
}

impl<F: Float + ToPrimitive + FromPrimitive> ClBuffer for LightVolume<F> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> () {
        buf.write_i32::<LittleEndian>(self.nx as i32).unwrap();
        buf.write_i32::<LittleEndian>(self.ny as i32).unwrap();
        buf.write_i32::<LittleEndian>(self.nz as i32).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.dx).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.dy).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.dy).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.offset_x).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.offset_y).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.offset_z).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.wx()).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.wy()).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.wz()).unwrap()).unwrap();
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for LightVolume<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let nx = map.get("nx");
        let ny = map.get("ny");
        let nz = map.get("nz");
        let dx = map.get("dx");
        let dy = map.get("dy");
        let dz = map.get("dz");
        let offset_x = map.get("offset_x");
        let offset_y = map.get("offset_y");
        let offset_z = map.get("offset_z");

        match (nx, ny, nz, dx, dy, dz, offset_x, offset_y, offset_z) {
            (Some(&Value::Integer(nx)),
             Some(&Value::Integer(ny)),
             Some(&Value::Integer(nz)),
             Some(&Value::Float(dx)),
             Some(&Value::Float(dy)),
             Some(&Value::Float(dz)),
             Some(&Value::Float(offset_x)),
             Some(&Value::Float(offset_y)),
             Some(&Value::Float(offset_z))) => Some(LightVolume{
                nx: nx as usize,
                ny: ny as usize,
                nz: nz as usize,
                dx: F::from_f64(dx).unwrap(),
                dy: F::from_f64(dy).unwrap(),
                dz: F::from_f64(dz).unwrap(),
                offset_x: F::from_f64(offset_x).unwrap(),
                offset_y: F::from_f64(offset_y).unwrap(),
                offset_z: F::from_f64(offset_z).unwrap(),
            }),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("nx".to_string(), Value::Integer(self.nx as i64));
        tr.insert("ny".to_string(), Value::Integer(self.ny as i64));
        tr.insert("nz".to_string(), Value::Integer(self.nz as i64));
        tr.insert("dx".to_string(), Value::Float(F::to_f64(&self.dx).unwrap()));
        tr.insert("dy".to_string(), Value::Float(F::to_f64(&self.dy).unwrap()));
        tr.insert("dz".to_string(), Value::Float(F::to_f64(&self.dz).unwrap()));
        tr.insert("offset_x".to_string(), Value::Float(F::to_f64(&self.offset_x).unwrap()));
        tr.insert("offset_y".to_string(), Value::Float(F::to_f64(&self.offset_y).unwrap()));
        tr.insert("offset_z".to_string(), Value::Float(F::to_f64(&self.offset_z).unwrap()));
        tr
    }
}

#[test]
fn test_light_volume() {
    let test = r#"
        nx = 100
        ny = 200
        nz = 300
        dx = 3.0
        dy = 2.0
        dz = 1.0
        offset_x = 4.0
        offset_y = 8.0
        offset_z = 12.0
    "#;

    let mut parser = Parser::new(test);
    let map = parser.parse().unwrap();
    let v: LightVolume<f32> = LightVolume::from_map(&map).unwrap();

    assert_eq!(v.nx, 100);
    assert_eq!(v.ny, 200);
    assert_eq!(v.nz, 300);
    assert_eq!(v.dx, 3.0);
    assert_eq!(v.dy, 2.0);
    assert_eq!(v.dz, 1.0);
    assert_eq!(v.offset_x, 4.0);
    assert_eq!(v.offset_y, 8.0);
    assert_eq!(v.offset_z, 12.0);

    let vv: LightVolume<f32> = LightVolume::from_map(&v.into_map()).unwrap();

    assert_eq!(v.nx, vv.nx);
    assert_eq!(v.ny, vv.ny);
    assert_eq!(v.nz, vv.nz);
    assert_eq!(v.dx, vv.dx);
    assert_eq!(v.dy, vv.dy);
    assert_eq!(v.dz, vv.dz);
    assert_eq!(v.offset_x, vv.offset_x);
    assert_eq!(v.offset_y, vv.offset_y);
    assert_eq!(v.offset_z, vv.offset_z);
}

