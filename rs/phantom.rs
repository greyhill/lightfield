extern crate num;
extern crate toml;

use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;

/// Ellipsoidal phantom
#[derive(Copy, Clone)]
pub struct Ellipsoid<F: Float> {
    pub xx: F,
    pub xy: F,
    pub xz: F,
    pub xr: F,

    pub yx: F,
    pub yy: F,
    pub yz: F,
    pub yr: F,

    pub zx: F,
    pub zy: F,
    pub zz: F,
    pub zr: F,

    pub value: F,
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for Ellipsoid<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let xx = map.get("xx");
        let xy = map.get("xy");
        let xz = map.get("xz");
        let xr = map.get("xr");

        let yx = map.get("yx");
        let yy = map.get("yy");
        let yz = map.get("yz");
        let yr = map.get("yr");

        let zx = map.get("zx");
        let zy = map.get("zy");
        let zz = map.get("zz");
        let zr = map.get("zr");

        let value = map.get("value");

        match (xx, xy, xz, xr,
               yx, yy, yz, yr,
               zx, zy, zz, zr,
               value) {
            (Some(&Value::Float(xx)), Some(&Value::Float(xy)), Some(&Value::Float(xz)), Some(&Value::Float(xr)),
             Some(&Value::Float(yx)), Some(&Value::Float(yy)), Some(&Value::Float(yz)), Some(&Value::Float(yr)),
             Some(&Value::Float(zx)), Some(&Value::Float(zy)), Some(&Value::Float(zz)), Some(&Value::Float(zr)),
             Some(&Value::Float(value))) => 
                Some(Ellipsoid{
                    xx: F::from_f64(xx).unwrap(),
                    xy: F::from_f64(xy).unwrap(),
                    xz: F::from_f64(xz).unwrap(),
                    xr: F::from_f64(xr).unwrap(),

                    yx: F::from_f64(yx).unwrap(),
                    yy: F::from_f64(yy).unwrap(),
                    yz: F::from_f64(yz).unwrap(),
                    yr: F::from_f64(yr).unwrap(),

                    zx: F::from_f64(zx).unwrap(),
                    zy: F::from_f64(zy).unwrap(),
                    zz: F::from_f64(zz).unwrap(),
                    zr: F::from_f64(zr).unwrap(),

                    value: F::from_f64(value).unwrap(),
                }),
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("xx".to_string(), Value::Float(F::to_f64(&self.xx).unwrap()));
        tr.insert("xy".to_string(), Value::Float(F::to_f64(&self.xy).unwrap()));
        tr.insert("xz".to_string(), Value::Float(F::to_f64(&self.xz).unwrap()));
        tr.insert("xr".to_string(), Value::Float(F::to_f64(&self.xr).unwrap()));

        tr.insert("yx".to_string(), Value::Float(F::to_f64(&self.yx).unwrap()));
        tr.insert("yy".to_string(), Value::Float(F::to_f64(&self.yy).unwrap()));
        tr.insert("yz".to_string(), Value::Float(F::to_f64(&self.yz).unwrap()));
        tr.insert("yr".to_string(), Value::Float(F::to_f64(&self.yr).unwrap()));

        tr.insert("zx".to_string(), Value::Float(F::to_f64(&self.zx).unwrap()));
        tr.insert("zy".to_string(), Value::Float(F::to_f64(&self.zy).unwrap()));
        tr.insert("zz".to_string(), Value::Float(F::to_f64(&self.zz).unwrap()));
        tr.insert("zr".to_string(), Value::Float(F::to_f64(&self.zr).unwrap()));

        tr.insert("value".to_string(), Value::Float(F::to_f64(&self.value).unwrap()));

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

        yx = 5.0
        yy = 6.0
        yz = 7.0
        yr = 8.0

        zx = 9.0
        zy = 10.0
        zz = 11.0
        zr = 12.0

        value = 13.0

        [[ellipsoid]]
        xx = 21.0
        xy = 22.0
        xz = 23.0
        xr = 24.0

        yx = 25.0
        yy = 26.0
        yz = 27.0
        yr = 28.0

        zx = 29.0
        zy = 210.0
        zz = 211.0
        zr = 212.0

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

            assert_eq!(e1.yx, 5.0);
            assert_eq!(e1.yy, 6.0);
            assert_eq!(e1.yz, 7.0);
            assert_eq!(e1.yr, 8.0);

            assert_eq!(e1.zx, 9.0);
            assert_eq!(e1.zy, 10.0);
            assert_eq!(e1.zz, 11.0);
            assert_eq!(e1.zr, 12.0);

            assert_eq!(e1.value, 13.0);

            assert_eq!(e2.xx, 21.0);
            assert_eq!(e2.xy, 22.0);
            assert_eq!(e2.xz, 23.0);
            assert_eq!(e2.xr, 24.0);

            assert_eq!(e2.yx, 25.0);
            assert_eq!(e2.yy, 26.0);
            assert_eq!(e2.yz, 27.0);
            assert_eq!(e2.yr, 28.0);

            assert_eq!(e2.zx, 29.0);
            assert_eq!(e2.zy, 210.0);
            assert_eq!(e2.zz, 211.0);
            assert_eq!(e2.zr, 212.0);

            assert_eq!(e2.value, 213.0);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}

