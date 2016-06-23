extern crate num;
extern crate proust;
use self::num::{FromPrimitive, ToPrimitive, Float};
use self::proust::*;
use detector::*;
use lens::*;
use mask::*;
use geom::*;
use bounding_geometry::*;
use occluder::*;
use transport::*;
use light_field_geom::*;
use optics::*;
use vector_math::*;
use std::cmp::min;

/// Microlens array operation
///
/// For computational reasons, a microlens array is assumed to always be
/// the last element of an optical chain before the detector.
pub struct LensArray<F: Float + FromPrimitive> {
    mask: Mask<F>,
    mask_buf: Mem,
    vecmath: VectorMath<F>,
    xports: Vec<Transport<F>>,
}

impl<F: Float + FromPrimitive + ToPrimitive> LensArray<F> {
    pub fn new(array_lfg: LightFieldGeometry<F>,
               detector: Detector<F>,
               distance_detector_array: F,
               lenses: &[Lens<F>],
               queue: CommandQueue)
               -> Result<Self, Error> {
        let array_geometry = &array_lfg.geom;

        // create ulens array mask and transports for each ulens array
        let mut lens_mask = array_geometry.zeros();
        let mut xports = Vec::new();
        for lens in lenses.iter() {
            // get the pixels on the ulens plane that this lens touches
            let lens_geom = lens.bounding_geometry(1, 1);
            let (s0, s1, t0, t1) = lens_geom.spatial_bounds();
            let (is0, is1, it0, it1) = array_geometry.region_pixels(s0, s1, t0, t1);

            if is1 == is0 || it1 == it0 {
                continue;
            }

            // update mask
            for it in it0..it1 {
                for is in is0..is1 {
                    let (ss0, ss1, tt0, tt1) = array_geometry.pixel_bounds(is, it);
                    let mask_val = F::one() - lens.rasterize(ss0, ss1, tt0, tt1, 10); // TODO magic number
                    let mask_index = is + array_geometry.ns * it;

                    // in case over overlap between two lenses, use the maximum
                    // mask value
                    let current_mask_value = lens_mask[mask_index];
                    if current_mask_value < mask_val {
                        lens_mask[mask_index] = mask_val;
                    }
                }
            }

            // light field geometry on the detector behind this lens
            let lens_detector_lfg = LightFieldGeometry {
                geom: detector.image_geometry(),
                plane: array_lfg.plane.clone(),
                to_plane: Optics::translation(&distance_detector_array)
                              .then(&lens.optics())
                              .then(&array_lfg.to_plane),
            };

            // TODO use a less magical dilation
            let dilation_s = (is1 - is0) / 2;
            let dilation_t = (it1 - it0) / 2;

            let ds0 = if is0 < dilation_s {
                0
            } else {
                is0 - dilation_s
            };
            let ds1 = min(is1 + dilation_s, array_geometry.ns);

            let dt0 = if it0 < dilation_t {
                0
            } else {
                it0 - dilation_t
            };
            let dt1 = min(it1 + dilation_t, array_geometry.nt);

            let xport = try!(Transport::new(array_lfg.clone(),
                                            lens_detector_lfg,
                                            Some((is0, is1, it0, it1)), // source bounds
                                            Some((ds0, ds1, dt0, dt1)), // destination bounds
                                            false, // overwrite forw
                                            true, // overwrite back
                                            true, // conservative forw,
                                            true, // conservative back
                                            true, // onto detector
                                            queue.clone()));
            xports.push(xport);
        }

        let mask = try!(Mask::new(array_geometry.clone(), &lens_mask[..], queue.clone()));
        let mask_buf = try!(array_geometry.zeros_buf(&queue));

        let vecmath = try!(VectorMath::new(queue.clone()));

        Ok(LensArray {
            mask: mask,
            mask_buf: mask_buf,
            xports: xports,
            vecmath: vecmath,
        })
    }

    pub fn forw(self: &mut Self,
                view: &Mem,
                det: &mut Mem,
                ia: usize,
                wait_for: &[Event])
                -> Result<Event, Error> {
        let mut tmp = self.mask_buf.clone();

        // apply the microlens array mask
        let evt = try!(self.mask.apply_mask_to(view, &mut tmp, wait_for));
        let mut evts: Vec<Event> = Vec::new();
        for xi in self.xports.iter_mut() {
            let evt_i = try!(xi.forw(&tmp, det, ia, &[evt.clone()]));
            evts.push(evt_i);
        }

        // coalesce all the ulens forw() events by doing another (idempotent)
        // masking of the tmp buffer.  TODO -- this is a bit hacky
        self.mask.apply_mask(&mut tmp, &evts)
    }

    pub fn back(self: &mut Self,
                det: &Mem,
                view: &mut Mem,
                ia: usize,
                wait_for: &[Event])
                -> Result<Event, Error> {
        let np = self.mask.geom.dimension();
        let evt = try!(self.vecmath.set(np, view, F::zero(), wait_for));

        // backproject from all ulenses onto ulens plane
        let mut evts: Vec<Event> = Vec::new();
        for xi in self.xports.iter_mut() {
            let evt_i = try!(xi.back(det, view, ia, &[evt.clone()]));
            evts.push(evt_i);
        }

        // apply mask
        self.mask.apply_mask(view, &evts)
    }
}
