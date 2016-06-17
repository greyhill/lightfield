extern crate num;
extern crate toml;
use serialize::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::toml::*;
use lens::*;
use detector::*;
use optics::*;

/// Single lens camera
#[derive(Clone, Debug)]
pub struct SingleLensCamera<F: Float> {
    pub lens: Lens<F>,
    pub detector: Detector<F>,
    pub distance_detector_lens: F,
}

impl<F: Float + FromPrimitive + ToPrimitive> SingleLensCamera<F> {
    pub fn focus_at_distance(self: &mut Self, focus_distance: F) {
        let pre_optics = Optics::identity();
        let post_optics = self.lens.optics().then(&Optics::translation(&focus_distance));
        let (distance_s, distance_t) = Optics::focus_at_distance(&pre_optics, &post_optics);
        self.distance_detector_lens = (distance_s + distance_t) / (F::one() + F::one());
    }
}

impl<F: Float + FromPrimitive + ToPrimitive> Serialize for SingleLensCamera<F> {
    fn from_map(map: &Table) -> Option<Self> {
        let lens = map.get("lens");
        let detector = map.get("detector");
        let distance_detector_lens = map.get("distance_detector_lens");

        match (lens, detector, distance_detector_lens) {
            (Some(&Value::Table(ref lens_tab)), 
             Some(&Value::Table(ref det_tab)),
             Some(&Value::Float(distance_detector_lens))) => match (Lens::from_map(lens_tab), Detector::from_map(det_tab)) {
                (Some(lens), Some(det)) => {
                    Some(SingleLensCamera{
                        lens: lens,
                        detector: det,
                        distance_detector_lens: F::from_f64(distance_detector_lens).unwrap(),
                    })
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn into_map(self: &Self) -> Table {
        let mut tr = Table::new();
        tr.insert("lens".to_string(), Value::Table(self.lens.into_map()));
        tr.insert("detector".to_string(), Value::Table(self.detector.into_map()));
        tr.insert("distance_detector_lens".to_string(), Value::Float(F::to_f64(&self.distance_detector_lens).unwrap()));
        tr
    }
}

#[test]
fn test_read_camera() {
    let test = r#"
    distance_detector_lens = 32.0

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
    let camera: SingleLensCamera<f64> = SingleLensCamera::from_map(&map).unwrap();

    assert_eq!(camera.distance_detector_lens, 32.0);

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

