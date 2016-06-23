extern crate num;
extern crate toml;
extern crate byteorder;

use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use cl_traits::*;
use self::byteorder::*;

/// Ellipsoidal phantom
#[derive(Clone, Debug)]
pub struct Ellipsoid<F: Float> {
    pub xx: F,
    pub xy: F,
    pub xz: F,
    pub xr: F,
    pub xc: F,

    pub yx: F,
    pub yy: F,
    pub yz: F,
    pub yr: F,
    pub yc: F,

    pub zx: F,
    pub zy: F,
    pub zz: F,
    pub zr: F,
    pub zc: F,

    pub value: F,
}

impl<F: Float> Ellipsoid<F> {
    pub fn sphere(x: F, y: F, z: F, radius: F, value: F) -> Self {
        Ellipsoid {
            xx: F::one(),
            xy: F::zero(),
            xz: F::zero(),
            xr: radius.clone(),
            xc: x,

            yx: F::zero(),
            yy: F::one(),
            yz: F::zero(),
            yr: radius.clone(),
            yc: y,

            zx: F::zero(),
            zy: F::zero(),
            zz: F::one(),
            zr: radius.clone(),
            zc: z,

            value: value,
        }
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for Ellipsoid<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let xx = map.get("xx");
        let xy = map.get("xy");
        let xz = map.get("xz");
        let xr = map.get("xr");
        let xc = map.get("xc");

        let yx = map.get("yx");
        let yy = map.get("yy");
        let yz = map.get("yz");
        let yr = map.get("yr");
        let yc = map.get("yc");

        let zx = map.get("zx");
        let zy = map.get("zy");
        let zz = map.get("zz");
        let zr = map.get("zr");
        let zc = map.get("zc");

        let value = map.get("value");

        match (xx,
               xy,
               xz,
               xr,
               xc,
               yx,
               yy,
               yz,
               yr,
               yc,
               zx,
               zy,
               zz,
               zr,
               zc,
               value) {
            (Some(&Value::Float(xx)),
             Some(&Value::Float(xy)),
             Some(&Value::Float(xz)),
             Some(&Value::Float(xr)),
             Some(&Value::Float(xc)),
             Some(&Value::Float(yx)),
             Some(&Value::Float(yy)),
             Some(&Value::Float(yz)),
             Some(&Value::Float(yr)),
             Some(&Value::Float(yc)),
             Some(&Value::Float(zx)),
             Some(&Value::Float(zy)),
             Some(&Value::Float(zz)),
             Some(&Value::Float(zr)),
             Some(&Value::Float(zc)),
             Some(&Value::Float(value))) => {
                Some(Ellipsoid {
                    xx: F::from_f64(xx).unwrap(),
                    xy: F::from_f64(xy).unwrap(),
                    xz: F::from_f64(xz).unwrap(),
                    xr: F::from_f64(xr).unwrap(),
                    xc: F::from_f64(xc).unwrap(),

                    yx: F::from_f64(yx).unwrap(),
                    yy: F::from_f64(yy).unwrap(),
                    yz: F::from_f64(yz).unwrap(),
                    yr: F::from_f64(yr).unwrap(),
                    yc: F::from_f64(yc).unwrap(),

                    zx: F::from_f64(zx).unwrap(),
                    zy: F::from_f64(zy).unwrap(),
                    zz: F::from_f64(zz).unwrap(),
                    zr: F::from_f64(zr).unwrap(),
                    zc: F::from_f64(zc).unwrap(),

                    value: F::from_f64(value).unwrap(),
                })
            }
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("xx".to_string(), Value::Float(F::to_f64(&self.xx).unwrap()));
        tr.insert("xy".to_string(), Value::Float(F::to_f64(&self.xy).unwrap()));
        tr.insert("xz".to_string(), Value::Float(F::to_f64(&self.xz).unwrap()));
        tr.insert("xr".to_string(), Value::Float(F::to_f64(&self.xr).unwrap()));
        tr.insert("xc".to_string(), Value::Float(F::to_f64(&self.xc).unwrap()));

        tr.insert("yx".to_string(), Value::Float(F::to_f64(&self.yx).unwrap()));
        tr.insert("yy".to_string(), Value::Float(F::to_f64(&self.yy).unwrap()));
        tr.insert("yz".to_string(), Value::Float(F::to_f64(&self.yz).unwrap()));
        tr.insert("yr".to_string(), Value::Float(F::to_f64(&self.yr).unwrap()));
        tr.insert("yc".to_string(), Value::Float(F::to_f64(&self.yc).unwrap()));

        tr.insert("zx".to_string(), Value::Float(F::to_f64(&self.zx).unwrap()));
        tr.insert("zy".to_string(), Value::Float(F::to_f64(&self.zy).unwrap()));
        tr.insert("zz".to_string(), Value::Float(F::to_f64(&self.zz).unwrap()));
        tr.insert("zr".to_string(), Value::Float(F::to_f64(&self.zr).unwrap()));
        tr.insert("zc".to_string(), Value::Float(F::to_f64(&self.zc).unwrap()));

        tr.insert("value".to_string(),
                  Value::Float(F::to_f64(&self.value).unwrap()));

        tr
    }
}

impl<F: Float> ClHeader for Ellipsoid<F> {
    fn header() -> &'static str {
        include_str!("../cl/ellipsoid_f32.opencl")
    }
}

impl<F: Float + ToPrimitive> ClBuffer for Ellipsoid<F> {
    fn as_cl_bytes(self: &Self, buf: &mut Vec<u8>) -> () {
        buf.write_f32::<LittleEndian>(F::to_f32(&self.xx).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.xy).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.xz).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.xr).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.xc).unwrap()).unwrap();

        buf.write_f32::<LittleEndian>(F::to_f32(&self.yx).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.yy).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.yz).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.yr).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.yc).unwrap()).unwrap();

        buf.write_f32::<LittleEndian>(F::to_f32(&self.zx).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.zy).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.zz).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.zr).unwrap()).unwrap();
        buf.write_f32::<LittleEndian>(F::to_f32(&self.zc).unwrap()).unwrap();

        buf.write_f32::<LittleEndian>(F::to_f32(&self.value).unwrap()).unwrap();
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for Vec<Ellipsoid<F>> {
    fn from_map(map: &Table) -> Option<Self> {
        if let Some(&Value::Array(ref arr)) = map.get("ellipsoids") {
            let mut tr = Vec::new();
            for it in arr.iter() {
                if let &Value::Table(ref t) = it {
                    if let Some(e) = Ellipsoid::from_map(t) {
                        tr.push(e);
                    }
                }
            }
            Some(tr)
        } else {
            None
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        let mut vals: Vec<Value> = Vec::new();
        for it in self.iter() {
            let ellipsoid_table = it.into_map();
            vals.push(Value::Table(ellipsoid_table));
        }
        tr.insert("ellipsoids".to_string(), Value::Array(vals));
        tr
    }
}

#[test]
fn test_read_phantom() {
    let test = r#"
        [[ellipsoid]]
        xx = 1.0
        xy = 2.0
        xz = 3.0
        xr = 4.0
        xc = -1.0

        yx = 5.0
        yy = 6.0
        yz = 7.0
        yr = 8.0
        yc = -2.0

        zx = 9.0
        zy = 10.0
        zz = 11.0
        zr = 12.0
        zc = -3.0

        value = 13.0

        [[ellipsoid]]
        xx = 21.0
        xy = 22.0
        xz = 23.0
        xr = 24.0
        xc = -4.0

        yx = 25.0
        yy = 26.0
        yz = 27.0
        yr = 28.0
        yc = -5.0

        zx = 29.0
        zy = 210.0
        zz = 211.0
        zr = 212.0
        zc = -6.0

        value = 213.0
    "#;

    let mut parser = Parser::new(test);
    let map = parser.parse().unwrap();
    if let Some(&Value::Array(ref ellipsoid_tables)) = map.get("ellipsoid") {
        if let (&Value::Table(ref e0_tab), &Value::Table(ref e1_tab)) = (&ellipsoid_tables[0],
                                                                         &ellipsoid_tables[1]) {
            let e1 = Ellipsoid::<f32>::from_map(e0_tab).unwrap();
            let e2 = Ellipsoid::<f32>::from_map(e1_tab).unwrap();

            assert_eq!(e1.xx, 1.0);
            assert_eq!(e1.xy, 2.0);
            assert_eq!(e1.xz, 3.0);
            assert_eq!(e1.xr, 4.0);
            assert_eq!(e1.xc, -1.0);

            assert_eq!(e1.yx, 5.0);
            assert_eq!(e1.yy, 6.0);
            assert_eq!(e1.yz, 7.0);
            assert_eq!(e1.yr, 8.0);
            assert_eq!(e1.yc, -2.0);

            assert_eq!(e1.zx, 9.0);
            assert_eq!(e1.zy, 10.0);
            assert_eq!(e1.zz, 11.0);
            assert_eq!(e1.zr, 12.0);
            assert_eq!(e1.zc, -3.0);

            assert_eq!(e1.value, 13.0);

            assert_eq!(e2.xx, 21.0);
            assert_eq!(e2.xy, 22.0);
            assert_eq!(e2.xz, 23.0);
            assert_eq!(e2.xr, 24.0);
            assert_eq!(e2.xc, -4.0);

            assert_eq!(e2.yx, 25.0);
            assert_eq!(e2.yy, 26.0);
            assert_eq!(e2.yz, 27.0);
            assert_eq!(e2.yr, 28.0);
            assert_eq!(e2.yc, -5.0);

            assert_eq!(e2.zx, 29.0);
            assert_eq!(e2.zy, 210.0);
            assert_eq!(e2.zz, 211.0);
            assert_eq!(e2.zr, 212.0);
            assert_eq!(e2.zc, -6.0);

            assert_eq!(e2.value, 213.0);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}
