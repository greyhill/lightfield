extern crate num;
extern crate nalgebra;

use self::nalgebra::*;
use self::num::{Float, FromPrimitive};
use plenoptic_camera::*;
use lens_array::*;
use geom::*;

/// Calibration results for a plenoptic camera using a corresponding-points algorithm
#[derive(Clone, Debug)]
pub struct PlenopticCameraCalibration<F: Float + FromPrimitive> {
    pub distance_lens_array: F,
    pub point_distances: Vec<F>,
    pub point_locations: Vec<(F, F)>,
}

impl<F: BaseNum + Float + FromPrimitive> PlenopticCameraCalibration<F> {
    pub fn new(camera: &PlenopticCamera<F>,
               pixel_coords: &[Vec<(usize, usize)>],
               distance_estimates: &[F],
               calibrate_internal: bool,
               estimate_depth: bool) -> Option<Self> {
        let detector_ig = camera.detector.image_geometry();
        let lenses = if let Some(ref lenses) = camera.array {
            lenses
        } else {
            panic!("Must call calibrate_from_points with loaded microlens array");
        };

        // build map of pixels to microlens coordinates
        let microlens_map = LensArray::microlens_map(&detector_ig, lenses);
        let microlens_map_f: Vec<F> = microlens_map.iter().map(|&o| 
                                                               if let Some(p) = o {
                                                                   F::from_usize(p).unwrap()
                                                               } else {
                                                                   F::zero()
                                                               }).collect();

        // convert pixel indices to locations, compute (u, v) coordinates
        let mut spatial_coords = Vec::new();
        for (i_point, point_pixels) in pixel_coords.iter().enumerate() {
            let mut point_coords = Vec::new();
            for &(is, it) in point_pixels.iter() {
                let index = detector_ig.address_linear(is, it);
                let s = detector_ig.is2s(is);
                let t = detector_ig.it2t(it);

                if let Some(lens_index) = microlens_map[index] {
                    let lens = &lenses[lens_index];
                    let u = (lens.center_s - s)/camera.distance_detector_array;
                    let v = (lens.center_t - t)/camera.distance_detector_array;

                    // we advance each ray to the microlens array.  since we're tracing
                    // the ray through the center of each microlens (that's how we got the
                    // (u,v) coordinates), we just use the center coordinates for the lens.
                    point_coords.push((lens.center_s, lens.center_t, u, v));
                } else {
                    println!("Pixel ({}, {}) has no associated microlens", is, it);
                }
            }

            if point_coords.len() == 0 {
                println!("Point {} (of {}) had no valid pixel coordinates?",
                    i_point + 1, pixel_coords.len());
            } else {
                spatial_coords.push(point_coords);
            }
        }

        if spatial_coords.len() == 0 {
            println!("Found no points with valid coordinates; aborting calibration");
            return None;
        }

        // compute cost function components
        let fs = camera.lens.focal_length_s;
        let ft = camera.lens.focal_length_t;
        let cs = camera.lens.center_s;
        let ct = camera.lens.center_t;
        let mut cost_alphas = Vec::new();
        let mut cost_betas = Vec::new();
        for point_rays in spatial_coords.iter() {
            let mut point_alphas = Vec::new();
            let mut point_betas = Vec::new();

            let mut point_alphas_s = Vec::new();
            let mut point_alphas_t = Vec::new();

            let mut dpd_s_sum = F::zero();
            let mut dp_s_sum = F::zero();
            let mut d_s_sum = F::zero();
            let mut s_sum = F::zero();

            let mut dpd_t_sum = F::zero();
            let mut dp_t_sum = F::zero();
            let mut d_t_sum = F::zero();
            let mut t_sum = F::zero();

            for &(s, t, u, v) in point_rays.iter() {
                // s/u component
                let a_dpd_s = -u/fs;
                let a_dp_s = cs/fs - s/fs + u;
                let a_d_s = u;
                let a_s = s;
                point_alphas_s.push((a_dpd_s, a_dp_s, a_d_s, a_s));

                dpd_s_sum = dpd_s_sum + a_dpd_s;
                dp_s_sum = dp_s_sum + a_dp_s;
                d_s_sum = d_s_sum + a_d_s;
                s_sum = s_sum + a_s;

                // t/v component
                let a_dpd_t = -v/ft;
                let a_dp_t = ct/ft - t/ft + v;
                let a_d_t = v;
                let a_t = t;
                point_alphas_t.push((a_dpd_t, a_dp_t, a_d_t, a_t));

                dpd_t_sum = dpd_t_sum + a_dpd_t;
                dp_t_sum = dp_t_sum + a_dp_t;
                d_t_sum = d_t_sum + a_d_t;
                t_sum = t_sum + a_t;
            }

            let denom = F::from_usize(point_rays.len()).unwrap();
            dpd_s_sum = dpd_s_sum / denom;
            dp_s_sum = dp_s_sum / denom;
            d_s_sum = d_s_sum / denom;
            s_sum = s_sum / denom;

            dpd_t_sum = dpd_t_sum / denom;
            dp_t_sum = dp_t_sum / denom;
            d_t_sum = d_t_sum / denom;
            t_sum = t_sum / denom;

            for &(a_dpd_s, dp_s, a_d_s, a_s) in point_alphas_s.iter() {
                point_alphas.push((a_dpd_s, dp_s, a_d_s, a_s));
                point_betas.push((a_dpd_s - dpd_s_sum,
                                 dp_s - dp_s_sum,
                                 a_d_s - d_s_sum,
                                 a_s - s_sum));
            }

            for &(a_dpd_t, dp_t, a_d_t, a_t) in point_alphas_t.iter() {
                point_alphas.push((a_dpd_t, dp_t, a_d_t, a_t));
                point_betas.push((a_dpd_t - dpd_t_sum,
                                 dp_t - dp_t_sum,
                                 a_d_t - d_t_sum,
                                 a_t - t_sum));
            }

            cost_alphas.push(point_alphas);
            cost_betas.push(point_betas);
        }

        // initialize variables to calibrate
        let np = distance_estimates.len();
        let mut state_vector = DVector::from_element(np+1, F::zero());
        state_vector[0] = camera.distance_lens_array;
        for (n, d) in distance_estimates.iter().enumerate() {
            state_vector[n+1] = d.clone();
        }

        // optimization iterations
        // TODO add an actual stopping criterion
        for _ in 0 .. 2048 {
            let mut hessian = DMatrix::from_element(np+1, np+1, F::zero());
            let mut gradient = DVector::from_element(np+1, F::zero());
            let mut cost = F::zero();
            let d = state_vector[0];

            // compute forward components
            let mut forward = Vec::new();
            for (p, betas) in cost_betas.iter().enumerate() {
                let dp = state_vector[p+1];
                let mut point_forward = Vec::new();
                for &(b_dpd, b_dp, b_d, b) in betas.iter() {
                    let v = b_dpd * dp * d + b_dp * dp + b_d * d + b;
                    point_forward.push(v);
                    cost = cost + v.powi(2);
                }
                forward.push(point_forward);
            }

            // compute gradient and Hessian
            //  - d (internal distance) element
            let mut grad_d = F::zero();
            let mut hess_d = F::zero();
            for (p, (point_forward, point_betas)) in forward.iter().zip(cost_betas.iter()).enumerate() {
                let dp = state_vector[p+1];
                for (&f, &(b_dpd, _, b_d, _)) in point_forward.iter().zip(point_betas.iter()) {
                    grad_d = grad_d + f * (b_dpd*dp + b_d);
                    hess_d = hess_d + (b_dpd*dp + b_d).powi(2);
                }
            }
            gradient[0] = grad_d;
            hessian[(0,0)] = hess_d;

            //  - dp (external distance) elements
            for (p, (point_forward, point_betas)) in forward.iter().zip(cost_betas.iter()).enumerate() {
                let mut grad_dp = F::zero();
                let mut hess_off = F::zero();
                let mut hess_dp = F::zero();
                let dp = state_vector[p+1];

                for (&f, &(b_dpd, b_dp, b_d, _)) in point_forward.iter().zip(point_betas.iter()) {
                    grad_dp = grad_dp + f*(b_dpd*d + b_dp);
                    hess_dp = hess_dp + (b_dpd*d + b_dp).powi(2);
                    hess_off = hess_off + f*b_dpd + (b_dpd*dp + b_d)*(b_dpd*d + b_dp);
                }

                gradient[p+1] = grad_dp;
                hessian[(p+1, p+1)] = hess_dp + hess_off.abs();
                hessian[(0,0)] = hessian[(0,0)] + hess_off.abs();
            }

            // hold terms constant if requested
            if !calibrate_internal {
                gradient[0] = F::zero();
            }
            if !estimate_depth {
                for p in 0 .. np {
                    gradient[p+1] = F::zero();
                }
            }

            // update
            state_vector = state_vector 
                - hessian.inverse().expect("Error inverting calibration Hessian") * gradient;
        }

        Some(PlenopticCameraCalibration{
            distance_lens_array: state_vector[0],
            point_distances: state_vector[1..].to_owned(),
            point_locations: Vec::new(),
        })
    }
}

