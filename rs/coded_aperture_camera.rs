extern crate num;
extern crate toml;
use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use lens::*;
use detector::*;
use image_geom::*;

/// Description of a coded aperture camera
#[derive(Clone, Debug)]
pub struct CodedApertureCamera<F: Float> {
    pub lens: Lens<F>,
    pub detector: Detector<F>,
    pub mask_geometry: ImageGeometry<F>,
    pub distance_lens_mask: F,
    pub distance_detector_mask: F,
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for CodedApertureCamera<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let lens = map.get("lens");
        let detector = map.get("detector");
        let mask_geometry = map.get("mask_geometry");
        let distance_lens_mask = map.get("distance_lens_mask");
        let distance_detector_mask = map.get("distance_detector_mask");

        match (lens, detector, mask_geometry, distance_lens_mask, distance_detector_mask) {
            (Some(&Value::Table(ref lens_tab)),
             Some(&Value::Table(ref det_tab)),
             Some(&Value::Table(ref mask_tab)),
             Some(&Value::Float(distance_lens_mask)),
             Some(&Value::Float(distance_detector_mask))) => {
                match (Lens::from_map(lens_tab), Detector::from_map(det_tab), ImageGeometry::from_map(mask_tab)) {
                    (Some(lens), Some(det), Some(mask_geom)) => {
                        Some(CodedApertureCamera{
                            lens: lens,
                            detector: det,
                            mask_geometry: mask_geom,
                            distance_lens_mask: F::from_f64(distance_lens_mask).unwrap(),
                            distance_detector_mask: F::from_f64(distance_detector_mask).unwrap(),
                        })
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("lens".to_string(), Value::Table(self.lens.into_map()));
        tr.insert("detector".to_string(), Value::Table(self.detector.into_map()));
        tr.insert("mask_geometry".to_string(), Value::Table(self.mask_geometry.into_map()));
        tr.insert("distance_lens_mask".to_string(), Value::Float(F::to_f64(&self.distance_lens_mask).unwrap()));
        tr.insert("distance_detector_mask".to_string(), Value::Float(F::to_f64(&self.distance_detector_mask).unwrap()));
        tr
    }
}

#[test]
fn test_coded_aperture() {
    let test = r#"
    distance_lens_mask = 32.0
    distance_detector_mask = 2.0

    [mask_geometry]
    ns = 300
    nt = 200
    ds = 1e-2
    dt = 1e-3
    offset_s = 0.5
    offset_t = 0.25

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
    let camera: CodedApertureCamera<f32> = CodedApertureCamera::from_map(&map).unwrap();

    assert_eq!(camera.distance_lens_mask, 32.0);
    assert_eq!(camera.distance_detector_mask, 2.0);

    assert_eq!(camera.mask_geometry.ns, 300);
    assert_eq!(camera.mask_geometry.nt, 200);
    assert_eq!(camera.mask_geometry.ds, 1e-2);
    assert_eq!(camera.mask_geometry.dt, 1e-3);
    assert_eq!(camera.mask_geometry.offset_s, 0.5);
    assert_eq!(camera.mask_geometry.offset_t, 0.25);

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

