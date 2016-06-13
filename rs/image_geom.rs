extern crate num;
extern crate toml;
extern crate byteorder;
extern crate avsfld;
extern crate image;
use serialize::*;
use cl_traits::*;
use geom::*;
use self::toml::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::byteorder::*;
use std::path::Path;
use std::fs::File;
use self::image::*;

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

fn fmin<F: Float>(x: F, y: F) -> F {
    if x < y {
        x
    } else {
        y
    }
}

fn fmax<F: Float>(x: F, y: F) -> F {
    if x > y {
        x
    } else {
        y
    }
}

impl<F: Float + ToPrimitive + FromPrimitive> ImageGeometry<F> {
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

    /// Returns (s0, s1, t0, t1) sptial bounds for this geometry
    pub fn spatial_bounds(self: &Self) -> (F, F, F, F) {
        let (s0, t0, _, _) = self.pixel_bounds(0, 0);
        let (_, _, s1, t1) = self.pixel_bounds(self.ns - 1, self.nt - 1);
        (s0, s1, t0, t1)
    }

    pub fn is2s(self: &Self, is: usize) -> F {
        (F::from_usize(is).unwrap() - self.ws())*self.ds
    }

    pub fn it2t(self: &Self, it: usize) -> F {
        (F::from_usize(it).unwrap() - self.wt())*self.dt
    }

    pub fn s2is(self: &Self, s: F) -> F {
        s/self.ds + self.ws() + F::from_f32(0.5f32).unwrap()
    }

    pub fn t2it(self: &Self, t: F) -> F {
        t/self.dt + self.wt() + F::from_f32(0.5f32).unwrap()
    }

    /// Returns (is0, is1, it0, it1) bounds for the pixel rectangular region
    /// bounding the given spatial coordinates
    pub fn region_pixels(self: &Self, s0: F, s1: F, t0: F, t1: F) -> (usize, usize, usize, usize) {
        let ns = F::from_usize(self.ns).unwrap();
        let nt = F::from_usize(self.nt).unwrap();
        let is0 = fmax(F::zero(), fmin(self.s2is(s0), ns));
        let is1 = fmax(F::zero(), fmin(self.s2is(s1), ns));
        let it0 = fmax(F::zero(), fmin(self.t2it(t0), nt));
        let it1 = fmax(F::zero(), fmin(self.t2it(t1), nt));
        (F::to_usize(&is0).unwrap(),
         F::to_usize(&is1).unwrap(),
         F::to_usize(&it0).unwrap(),
         F::to_usize(&it1).unwrap())
    }

    fn save_fld<P: AsRef<Path>>(self: &Self, buf: &[F], path: P) -> Result<(), ()> {
        let mut file = if let Ok(f) = File::create(path) {
            f
        } else {
            return Err(());
        };
        let sizes = [self.ns, self.nt];
        match self::avsfld::AVSFile::write(&mut file, &sizes, buf) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn load_fld<P: AsRef<Path>>(self: &Self, path: P) -> Result<Vec<F>, ()> {
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

    fn save_image<P: AsRef<Path>>(self: &Self, buf: &[F], path: P) -> Result<(), ()> {
        let mut min_val = buf[0].clone();
        let mut max_val = buf[0].clone();
        for &b in buf.iter() {
            if b < min_val {
                min_val = b.clone();
            }
            if b > max_val {
                max_val = b.clone();
            }
        }

        let image_f32 = buf.iter().map(|&f| {
            let v = (f - min_val)/(max_val - min_val)*F::from_u8(255).unwrap();
            F::to_u8(&v).unwrap()
        }).collect();

        let image: ImageBuffer<Luma<u8>, Vec<u8>> = 
            ImageBuffer::from_raw(self.ns as u32, self.nt as u32, image_f32)
                .expect("logic error -- buffer not big enough");
        match image.save(path) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn load_image<P: AsRef<Path>>(self: &Self, path: P) -> Result<Vec<F>, ()> {
        let dyn_image = if let Ok(dyn_image) = open(path) {
            dyn_image
        } else {
            return Err(());
        };
        if let DynamicImage::ImageLuma8(gray_image) = dyn_image.grayscale() {
            let raw = gray_image.into_raw();
            let tr = raw.into_iter().map(|byte| F::from_u8(byte).unwrap()).collect();
            Ok(tr)
        } else {
            panic!("Failed conversion to grayscale?");
        }
    }
}

impl<F: Float + FromPrimitive> Geometry<F> for ImageGeometry<F> {
    fn shape(self: &Self) -> Vec<usize> {
        vec![self.ns, self.nt]
    }

    fn save<P: AsRef<Path>>(self: &Self, buf: &[F], path: P) -> Result<(), ()> {
        if let Some(extension) = path.as_ref().extension() {
            match extension.to_str() {
                Some("fld") => self.save_fld(buf, &path),
                Some("png") => self.save_image(buf, &path),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    fn load<P: AsRef<Path>>(self: &Self, path: P) -> Result<Vec<F>, ()> {
        if let Some(extension) = path.as_ref().extension() {
            match extension.to_str() {
                Some("fld") => self.load_fld(&path),
                Some("bmp") | Some("png") | Some("tif") | Some("tiff") => self.load_image(&path),
                _ => Err(()),
            }
        } else {
            Err(())
        }
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

