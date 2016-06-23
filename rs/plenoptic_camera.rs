extern crate num;
extern crate toml;
use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use lens::*;
use detector::*;
use std::path::Path;
use scene::*;
use optics::*;

#[derive(Clone, Debug)]
pub struct PlenopticCamera<F: Float> {
    pub lens: Lens<F>,
    pub detector: Detector<F>,
    pub distance_lens_array: F,
    pub distance_detector_array: F,
    pub array_path: String,
    pub array: Option<Vec<Lens<F>>>,
}

impl<F: Float + FromPrimitive> PlenopticCamera<F> {
    pub fn focus_at_distance(self: &mut Self, focus_distance: F) {
        let mut distance = F::zero();
        let mut denom = F::zero();
        if let Some(ref array) = self.array {
            for lens in array.iter() {
                let pre_optics = Optics::translation(&self.distance_detector_array)
                                     .then(&lens.optics());
                let post_optics = self.lens.optics().then(&Optics::translation(&focus_distance));
                let (distance_s, distance_t) = Optics::focus_at_distance(&pre_optics, &post_optics);
                distance = distance + distance_s + distance_t;
                denom = denom + F::one() + F::one();
            }
        } else {
            panic!("Must call PlenopticCamera::focus_at_distance with loaded lens array");
        }
        self.distance_lens_array = distance / denom;
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for PlenopticCamera<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let lens = map.get("lens");
        let detector = map.get("detector");
        let distance_lens_array = map.get("distance_lens_array");
        let distance_detector_array = map.get("distance_detector_array");
        let array_path = map.get("array_path");

        match (lens,
               detector,
               distance_lens_array,
               distance_detector_array,
               array_path) {
            (Some(&Value::Table(ref lens_tab)),
             Some(&Value::Table(ref detector_tab)),
             Some(&Value::Float(distance_lens_array)),
             Some(&Value::Float(distance_detector_array)),
             Some(&Value::String(ref array_path))) => {
                match (Lens::from_map(lens_tab), Detector::from_map(detector_tab)) {
                    (Some(lens), Some(detector)) => {
                        Some(PlenopticCamera {
                            lens: lens,
                            detector: detector,
                            distance_lens_array: F::from_f64(distance_lens_array).unwrap(),
                            distance_detector_array: F::from_f64(distance_detector_array).unwrap(),
                            array_path: array_path.clone(),
                            array: None,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("lens".to_string(), Value::Table(self.lens.into_map()));
        tr.insert("detector".to_string(),
                  Value::Table(self.detector.into_map()));
        tr.insert("distance_lens_array".to_string(),
                  Value::Float(F::to_f64(&self.distance_lens_array).unwrap()));
        tr.insert("distance_detector_array".to_string(),
                  Value::Float(F::to_f64(&self.distance_detector_array).unwrap()));
        tr.insert("array_path".to_string(),
                  Value::String(self.array_path.clone()));
        tr
    }

    fn load_assets<P: AsRef<Path>>(self: &mut Self, root_path: P) -> Result<(), ()> {
        if self.array.is_none() {
            let mut pb = root_path.as_ref().to_path_buf();
            pb.pop(); // pop off configuration file name
            pb.push(&self.array_path);
            match table_from_file(&pb) {
                Some(tab) => {
                    match Vec::<Lens<F>>::from_map(&tab) {
                        Some(array) => {
                            self.array = Some(array);
                            Ok(())
                        }
                        None => Err(()),
                    }
                }
                None => Err(()),
            }
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_plenoptic_camera() {
    let test = r#"
    distance_lens_array = 32.0
    distance_detector_array = 1.0
    array_path = "../lenses/array.toml"

    [detector]
    ns = 1024
    nt = 2048
    ds = 5e-2
    dt = 5e-3
    offset_s = 1.0
    offset_t = 2.0

    [lens]
    center_s = 3.0
    center_t = -5.2
    radius_s = 4.0
    radius_t = 5.2
    focal_length_s = 12.0
    focal_length_t = 24.0
    "#;

    let map = Parser::new(test).parse().unwrap();
    let camera: PlenopticCamera<f32> = PlenopticCamera::from_map(&map).unwrap();

    assert_eq!(camera.distance_lens_array, 32.0);
    assert_eq!(camera.distance_detector_array, 1.0);

    assert_eq!(camera.detector.ns, 1024);
    assert_eq!(camera.detector.nt, 2048);
    assert_eq!(camera.detector.ds, 5e-2);
    assert_eq!(camera.detector.dt, 5e-3);
    assert_eq!(camera.detector.offset_s, 1.0);
    assert_eq!(camera.detector.offset_t, 2.0);

    assert_eq!(camera.lens.center_s, 3.0);
    assert_eq!(camera.lens.center_t, -5.2);
    assert_eq!(camera.lens.radius_s, 4.0);
    assert_eq!(camera.lens.radius_t, 5.2);
    assert_eq!(camera.lens.focal_length_s, 12.0);
    assert_eq!(camera.lens.focal_length_t, 24.0);
}
