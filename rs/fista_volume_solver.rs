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
use potential_function::*;
use cl_traits::*;
use optics::*;
use image_geom::*;

/// Translucent volume reconstruction via FISTA
pub struct FistaVolumeSolver<F: Float + FromPrimitive + ToPrimitive + BaseFloat> {
    geom: LightVolume<F>,
    imagers: Vec<Box<Imager<F, LightVolume<F>>>>,
    vecmath: VectorMath<F>,

    x: Mem,
    m: Mem,
    denom: Mem,
    measurements: Vec<Mem>,
    projections: Vec<Mem>,
    tmp_buffers: Vec<Mem>,

    camera_scales: Vec<F>,
    measurements_host: Vec<Vec<F>>,
    ynorm2s: Vec<F>,

    update: Kernel,
    sparsifying_buf: Option<Mem>,
    geom_buf: Mem,
    box_min: Option<F>,
    box_max: Option<F>,

    queue: CommandQueue,

    t: F,
}

impl<F: Float + FromPrimitive + ToPrimitive + BaseFloat> ClHeader for FistaVolumeSolver<F> {
    fn header() -> &'static str {
        include_str!("../cl/fista_volume_solver_f32.opencl")
    }
}

impl<F: Float + FromPrimitive + ToPrimitive + BaseFloat> FistaVolumeSolver<F> {
    pub fn new(geometry: LightVolume<F>,
               imagers: Vec<Box<Imager<F, LightVolume<F>>>>,
               measurements: &[&[F]],
               sparsifying_regularizer: &Option<PotentialFunction<F>>,
               box_min: Option<F>,
               box_max: Option<F>,
               queue: CommandQueue) -> Result<Self, Error> {
        // get opencl objects
        let context = try!(queue.context());
        let device = try!(queue.device());
        let sources = &[
            Optics::<F>::header(),
            ImageGeometry::<F>::header(),
            LightVolume::<F>::header(),
            PotentialFunction::<F>::header(),
            Self::header(),
        ];

        // build opencl kernels
        let unbuilt = try!(Program::new_from_source(context, sources));
        let built = try!(unbuilt.build(&[device]));

        let update = try!(built.create_kernel("FistaVolumeSolver_update"));

        // gather measurements onto gpu
        let mut measurements_vec = Vec::new();
        for &m in measurements.iter() {
            let m_buf = try!(queue.create_buffer_from_slice(m));
            measurements_vec.push(m_buf);
        }

        // compute ynorm2s
        // (this is used for multi-camera normalization)
        let mut ynorm2s = Vec::new();
        let mut measurements_host = Vec::new();
        for &m in measurements.iter() {
            ynorm2s.push(m.iter().fold(F::zero(), |l, &r| l + r*r));
            measurements_host.push(m.to_owned());
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
        let m = try!(geometry.zeros_buf(&queue));
        let denom = try!(geometry.zeros_buf(&queue));

        // create sparsifying buffer
        let sparsifying_buf = if let &Some(ref pf) = sparsifying_regularizer {
            Some(try!(pf.as_cl_buffer(&queue)))
        } else {
            None
        };

        // create geometry buffer
        let geom_buf = try!(geometry.as_cl_buffer(&queue));

        let mut volume_solver = FistaVolumeSolver{
            geom: geometry,
            imagers: imagers,
            vecmath: vecmath,

            x: x,
            m: m,
            denom: denom,
            tmp_buffers: tmp_buffers,
            measurements: measurements_vec,
            projections: projections,

            update: update,

            sparsifying_buf: sparsifying_buf,
            geom_buf: geom_buf,
            box_min: box_min,
            box_max: box_max,

            camera_scales: Vec::new(),
            measurements_host: measurements_host,
            ynorm2s: ynorm2s,

            queue: queue,

            t: F::one()
        };
        try!(volume_solver.compute_denominator());

        Ok(volume_solver)
    }

    /// Computes data-fidelity term diagonal majorizer and camera normalization
    /// factors
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

            self.camera_scales.push(F::one());

            // backproject
            evt = try!(imager.back(proj_buf, &mut tmp, &[evt]));

            // accumulate backprojected ones onto denom
            // note: we scale by camera_scale^2
            evt = try!(self.vecmath.mix(np_geom,
                                        &tmp,
                                        &self.denom,
                                        F::one(),
                                        F::one(),
                                        &mut denom_copy,
                                        &[evt]));

            try!(evt.wait());
        }

        // keep all entries within 1000 of one another
        let mut denom_host = self.geom.zeros();
        try!(self.queue.read_buffer(&self.denom, &mut denom_host));
        let max_val = denom_host.iter().fold(F::one(), |l, &r| if l > r { l } else { r });
        let c1000 = F::from_f32(1000f32).unwrap();
        for m in denom_host.iter_mut() {
            if *m < max_val / c1000 {
                *m = max_val / c1000;
            }
        }
        try!(self.queue.write_buffer(&mut self.denom, &denom_host));

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

        if camera > 0 {
            // for all cameras but the first, update the camera_scale
            try!(evt.wait());
            let mut proj_host = vec![F::zero(); np_det];
            try!(try!(self.queue.read_buffer(proj, &mut proj_host)).wait());
            let iprod = proj_host.iter().zip(self.measurements_host[camera].iter()).fold(
                F::zero(), |l, (&a, &b)| l + a*b);
            self.camera_scales[camera] = iprod / self.ynorm2s[camera];
            println!("Camera {} scale: {}", camera, 
                     F::to_f32(&self.camera_scales[camera]).unwrap());
        }

        // compute subset scaling
        let subset_scaling = F::from_usize(imager.na()).unwrap() / F::from_usize(subset_angles.len()).unwrap();
        let scaling = subset_scaling;

        // compute residual
        // note: we use scaling factors subset_scaling^2 on the projection and -subset_scaling on
        // the measurements:
        //          subset_gradient = scaling * A_subset' * ( scaling * A_subset * x - y )
        // this is a little bit different from x-ray ct
        evt = try!(self.vecmath.mix(np_det,
                                    proj,
                                    meas,
                                    scaling*scaling,
                                    -scaling*self.camera_scales[camera],
                                    &mut proj_copy,
                                    &[evt]));

        // clear tmp
        evt = try!(self.vecmath.set(np_obj, tmp, F::zero(), &[evt]));

        // backproject residual into tmp
        evt = try!(imager.back_subset(proj, tmp, subset_angles, &[evt]));

        Ok(evt)
    }

    fn update_image(self: &mut Self, wait_for: &[Event]) -> Result<Event, Error> {
        // compute t1
        let c2 = F::from_f32(2f32).unwrap();
        let c4 = F::from_f32(4f32).unwrap();
        let t1 = (F::one() + (F::one() + c4*self.t*self.t).sqrt())/c2;

        // bind arguments
        try!(self.update.bind(0, &self.geom_buf));
        match &self.sparsifying_buf {
            &Some(ref buf) => try!(self.update.bind(1, buf)),
            &None => try!(self.update.bind_null(1)),
        };
        try!(self.update.bind(2, &self.x));
        try!(self.update.bind(3, &self.denom));
        try!(self.update.bind(4, &self.tmp_buffers[0]));
        match self.box_min {
            Some(ref box_min) => try!(self.update.bind_scalar(5, &F::to_f32(box_min).unwrap())),
            None => try!(self.update.bind_scalar(5, &-F::infinity())),
        };
        match self.box_max {
            Some(ref box_max) => try!(self.update.bind_scalar(6, &F::to_f32(box_max).unwrap())),
            None => try!(self.update.bind_scalar(6, &F::infinity())),
        };
        try!(self.update.bind_mut(7, &mut self.m));
        try!(self.update.bind_scalar(8, &self.t));
        try!(self.update.bind_scalar(9, &t1));

        let local_size = (32, 8, 1);
        let global_size = (self.geom.nx, self.geom.ny, self.geom.nz);

        self.t = t1;

        self.queue.run_with_events(&mut self.update,
                                   local_size,
                                   global_size,
                                   wait_for)
    }

    /// Run one subset of the FISTA iteration using the given subset of 
    /// angles to compute the data-fidelity gradients
    pub fn run_subset(self: &mut Self,
                      subset_angles: &[usize],
                      wait_for: &[Event]) -> Result<Event, Error> {
        // compute the data gradient into self.tmp_buffers[0]
        let evt = try!(self.compute_data_gradient(subset_angles, wait_for));

        // update the image self.x
        self.update_image(&[evt])
    }

    pub fn image_buffer(self: &Self) -> Mem {
        self.x.clone()
    }
}

