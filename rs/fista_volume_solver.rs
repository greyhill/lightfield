extern crate nalgebra;
extern crate num;
extern crate proust;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::nalgebra::BaseFloat;
use light_volume::*;
use imager::*;
use vector_math::*;
use self::proust::*;
use geom::*;

/// Translucent volume reconstruction via FISTA
pub struct FistaVolumeSolver<F: Float + FromPrimitive + ToPrimitive + BaseFloat> {
    geom: LightVolume<F>,
    imagers: Vec<Box<Imager<F, LightVolume<F>>>>,
    vecmath: VectorMath<F>,

    x: Mem,
    denom: Mem,
    measurements: Vec<Mem>,
    projections: Vec<Mem>,
    tmp_buffers: Vec<Mem>,

    queue: CommandQueue,
}

impl<F: Float + FromPrimitive + ToPrimitive + BaseFloat> FistaVolumeSolver<F> {
    pub fn new(geometry: LightVolume<F>,
               imagers: Vec<Box<Imager<F, LightVolume<F>>>>,
               measurements: &[&[F]],
               queue: CommandQueue) -> Result<Self, Error> {
        // gather measurements onto gpu
        let mut measurements_vec = Vec::new();
        for &m in measurements.iter() {
            let m_buf = try!(queue.create_buffer_from_slice(m));
            measurements_vec.push(m_buf);
        }

        // create projection buffers
        let mut projections = Vec::new();
        let mut tmp_buffers = Vec::new();
        for imager in imagers.iter() {
            let det_geom = imager.detector().image_geometry();
            let proj_buf = try!(det_geom.zeros_buf(&queue));
            projections.push(proj_buf);

            let tmp_buf = try!(geometry.zeros_buf(&queue));
            tmp_buffers.push(tmp_buf);
        }

        // create vector math object
        let vecmath = try!(VectorMath::new(queue.clone()));

        // create blank x object
        let x = try!(geometry.zeros_buf(&queue));
        let denom = try!(geometry.zeros_buf(&queue));

        let mut volume_solver = FistaVolumeSolver{
            geom: geometry,
            imagers: imagers,
            vecmath: vecmath,

            x: x,
            denom: denom,
            tmp_buffers: tmp_buffers,
            measurements: measurements_vec,
            projections: projections,

            queue: queue,
        };
        try!(volume_solver.compute_denominator());

        Ok(volume_solver)
    }

    fn compute_denominator(self: &mut Self) -> Result<(), Error> {
        let ones = try!(self.geom.ones_buf(&self.queue));
        let mut tmp = try!(self.geom.zeros_buf(&self.queue));
        let mut denom_copy = self.denom.clone();

        let np_geom = self.geom.dimension();

        for (imager, proj_buf) in self.imagers.iter_mut().zip(self.projections.iter_mut()) {
            // clear tmp buf
            let mut evt = try!(self.vecmath.set(np_geom, &mut tmp, F::zero(), &[]));

            // project and backproject volume of ones into tmp
            evt = try!(imager.forw(&ones, proj_buf, &[evt]));
            evt = try!(imager.back(proj_buf, &mut tmp, &[evt]));

            // accumulate backprojected ones onto denom
            evt = try!(self.vecmath.mix(np_geom,
                                        &tmp,
                                        &self.denom,
                                        F::one(),
                                        F::one(),
                                        &mut denom_copy,
                                        &[evt]));

            try!(evt.wait());
        }
        Ok(())
    }

    /// computes the data gradient for all cameras and stores the accumulated
    /// gradient in `self.tmp_buffers[0]`
    fn compute_data_gradient(self: &mut Self,
                             subset_angles: &[usize],
                             wait_for: &[Event]) -> Result<Event, Error> {
        let num_cam = self.imagers.len();
        let np_obj = self.geom.dimension();

        // loop through cameras and compute gradient for each
        let mut camera_events = Vec::new();
        for camera in 0 .. num_cam {
            let evt = try!(self.compute_camera_gradient(camera, subset_angles, wait_for));
            camera_events.push(evt);
        }

        // accumulate gradients onto the tmp buffer for the first camera
        let mut evts_iter = camera_events.iter();
        let mut tmp_iter = self.tmp_buffers.iter_mut();
        let mut evt = evts_iter.next().unwrap().clone();
        let tmp0 = tmp_iter.next().unwrap();
        let mut tmp0_copy = tmp0.clone();
        for (evt_i, tmp_i) in evts_iter.zip(tmp_iter) {
            let wait = vec![evt, evt_i.clone()];
            evt = try!(self.vecmath.mix(np_obj,
                                        tmp0, tmp_i,
                                        F::one(), F::one(),
                                        &mut tmp0_copy,
                                        &wait));
        }

        Ok(evt)
    }

    fn compute_camera_gradient(self: &mut Self,
                               camera: usize,
                               subset_angles: &[usize],
                               wait_for: &[Event]) -> Result<Event, Error> {
        let imager = &mut self.imagers[camera];
        let tmp = &mut self.tmp_buffers[camera];
        let proj = &mut self.projections[camera];
        let meas = &mut self.measurements[camera];
        let mut proj_copy = proj.clone();

        let np_obj = self.geom.dimension();
        let np_det = imager.detector().image_geometry().dimension();

        // clear proj buffer
        let mut evt = try!(self.vecmath.set(np_det, proj, F::zero(), wait_for));

        // project x
        evt = try!(imager.forw_subset(&self.x,
                                      proj,
                                      subset_angles,
                                      &[evt]));

        // compute subset scaling
        let subset_scaling = F::from_usize(imager.na()).unwrap() / F::from_usize(subset_angles.len()).unwrap();

        // compute residual
        // note: we use scaling factors subset_scaling^2 on the projection and -subset_scaling on
        // the measurements:
        //          subset_gradient = scaling * A_subset' * ( scaling A_subset * x - y )
        // this is a little bit different from x-ray ct
        evt = try!(self.vecmath.mix(np_det,
                                    proj,
                                    meas,
                                    subset_scaling*subset_scaling,
                                    -subset_scaling,
                                    &mut proj_copy,
                                    &[evt]));

        // clear tmp
        evt = try!(self.vecmath.set(np_obj, tmp, F::zero(), &[evt]));

        // backproject residual into tmp
        evt = try!(imager.back_subset(proj, tmp, subset_angles, &[evt]));

        Ok(evt)
    }

    /// Run one subset of the FISTA iteration using the given subset of 
    /// angles to compute the data-fidelity gradients
    pub fn run_subset(self: &mut Self,
                      subset_angles: &[usize],
                      wait_for: &[Event]) -> Result<Event, Error> {
        let np_obj = self.geom.dimension();

        // compute the data gradient into self.tmp_buffers[0]
        let mut evt = try!(self.compute_data_gradient(subset_angles, wait_for));
        let mut grad_buf = self.tmp_buffers[0].clone();

        // TODO -- add regularization and momentum
        // scale gradient by self.denom
        evt = try!(self.vecmath.div(np_obj, &self.tmp_buffers[0], &self.denom, &mut grad_buf, &[evt]));

        // add to current image
        // x+ = x- - g
        let mut x_copy = self.x.clone();
        self.vecmath.mix(np_obj, &self.x, &grad_buf, F::one(), -F::one(), &mut x_copy, &[evt])
    }

    pub fn image_buffer(self: &Self) -> Mem {
        self.x.clone()
    }
}

