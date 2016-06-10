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

