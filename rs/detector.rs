extern crate num;
extern crate toml;
use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use image_geom::*;

/// Photodetector
#[derive(Clone, Debug)]
pub struct Detector<F: Float> {
    pub ns: usize,
    pub nt: usize,
    pub ds: F,
    pub dt: F,
    pub offset_s: F,
    pub offset_t: F,
}

impl<F: Float> Detector<F> {
    pub fn image_geometry(self: &Self) -> ImageGeometry<F> {
        ImageGeometry{
            ns: self.ns,
            nt: self.nt,
            ds: self.ds,
            dt: self.dt,
            offset_s: self.offset_s,
            offset_t: self.offset_t,
        }
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for Detector<F> {
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
             Some(&Value::Float(offset_t))) => Some(Detector{
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
fn test_read_detector() {
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
    let det: Detector<f64> = Detector::from_map(&map).unwrap();

    assert_eq!(det.ns, 1024);
    assert_eq!(det.nt, 2048);
    assert_eq!(det.ds, 5e-3);
    assert_eq!(det.dt, 2e-3);
    assert_eq!(det.offset_s, 0.1);
    assert_eq!(det.offset_t, 0.2);
}

