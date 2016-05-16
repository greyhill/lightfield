extern crate num;
extern crate toml;
extern crate byteorder;
use serialize::*;
use cl_traits::*;
use geom::*;
use self::toml::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::byteorder::*;

/// Pixel or plane geometry
#[derive(Clone, Debug)]
pub struct ImageGeometry<F: Float> {
    pub ns: usize,
    pub nt: usize,
    pub ds: F,
    pub dt: F,
    pub offset_s: F,
    pub offset_t: F,
}

impl<F: Float + FromPrimitive> ImageGeometry<F> {
    pub fn ws(self: &Self) -> F {
        (F::from_usize(self.ns).unwrap() - F::one())/F::from_f32(2f32).unwrap() + self.offset_s
    }

    pub fn wt(self: &Self) -> F {
        (F::from_usize(self.nt).unwrap() - F::one())/F::from_f32(2f32).unwrap() + self.offset_t
    }

    /// Returns pixel spatial bounds (s0, s1, t0, t1)
    pub fn pixel_bounds(self: &Self, is: usize, it: usize) -> (F, F, F, F) {
        let s = (F::from_usize(is).unwrap() - self.ws())*self.ds;
        let t = (F::from_usize(it).unwrap() - self.wt())*self.dt;
        let ds2 = self.ds / F::from_f32(2f32).unwrap();
        let dt2 = self.dt / F::from_f32(2f32).unwrap();
        (s - ds2, s + ds2, t - dt2, t + dt2)
    }

    /// Returns the spatial center of a pixel
    pub fn pixel_center(self: &Self, is: usize, it: usize) -> (F, F) {
        let s = (F::from_usize(is).unwrap() - self.ws())*self.ds;
        let t = (F::from_usize(it).unwrap() - self.wt())*self.dt;
        (s, t)
    }
}

impl<F: Float + FromPrimitive> Geometry<F> for ImageGeometry<F> {
    fn shape(self: &Self) -> Vec<usize> {
        vec![self.ns, self.nt]
    }
}

impl<F: Float> ClHeader for ImageGeometry<F> {
    fn header() -> &'static str {
        include_str!("../cl/image_geom_f32.opencl")
    }
}

impl<F: Float + ToPrimitive + FromPrimitive> ClBuffer for ImageGeometry<F> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> () {
        buf.write_i32::<LittleEndian>(self.ns as i32).unwrap();
        buf.write_i32::<LittleEndian>(self.nt as i32).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.ds).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.dt).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.offset_s).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.offset_t).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.ws()).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.wt()).unwrap()).unwrap();
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for ImageGeometry<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let ns = map.get("ns");
        let nt = map.get("nt");
        let ds = map.get("ds");
        let dt = map.get("dt");
        let offset_s = map.get("offset_s");
        let offset_t = map.get("offset_t");

        match (ns, nt, ds, dt, offset_s, offset_t) {
            (Some(&Value::Integer(ns)), 
             Some(&Value::Integer(nt)),
             Some(&Value::Float(ds)),
             Some(&Value::Float(dt)),
             Some(&Value::Float(offset_s)),
             Some(&Value::Float(offset_t))) => Some(ImageGeometry{
                ns: ns as usize,
                nt: nt as usize,
                ds: F::from_f64(ds).unwrap(),
                dt: F::from_f64(dt).unwrap(),
                offset_s: F::from_f64(offset_s).unwrap(),
                offset_t: F::from_f64(offset_t).unwrap(),
            }),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("ns".to_string(), Value::Integer(self.ns as i64));
        tr.insert("nt".to_string(), Value::Integer(self.nt as i64));
        tr.insert("ds".to_string(), Value::Float(F::to_f64(&self.ds).unwrap()));
        tr.insert("dt".to_string(), Value::Float(F::to_f64(&self.dt).unwrap()));
        tr.insert("offset_s".to_string(), Value::Float(F::to_f64(&self.offset_s).unwrap()));
        tr.insert("offset_t".to_string(), Value::Float(F::to_f64(&self.offset_t).unwrap()));
        tr
    }
}

#[test]
fn test_read_image_geom() {
    let test = r#"
        ns = 1024
        nt = 2048
        ds = 5e-3
        dt = 2e-3
        offset_s = 0.1
        offset_t = 0.2
    "#;

    let mut parser = Parser::new(test);
    let map = parser.parse().unwrap();
    let ig: ImageGeometry<f64> = ImageGeometry::from_map(&map).unwrap();

    assert_eq!(ig.ns, 1024);
    assert_eq!(ig.nt, 2048);
    assert_eq!(ig.ds, 5e-3);
    assert_eq!(ig.dt, 2e-3);
    assert_eq!(ig.offset_s, 0.1);
    assert_eq!(ig.offset_t, 0.2);
}

