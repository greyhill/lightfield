extern crate num;
extern crate proust;
extern crate nalgebra;
use light_volume::*;
use light_field_geom::*;
use angular_plane::*;
use optics::*;
use image_geom::*;
use isometry::*;
use cl_traits::*;
use self::num::{Float, FromPrimitive, ToPrimitive};
use self::proust::*;
use self::nalgebra::{Transformation, BaseFloat};

/// Renders light field volumes with ray tracing
pub struct LightVolumeTracer<'geom, 'lfg, F: 'geom + 'lfg + Float> {
    pub volume: &'geom LightVolume<F>,
    pub light_field: &'lfg LightFieldGeometry<F>,

    plane_to_geom: Isometry<F>,
    geom_to_plane: Isometry<F>,

    queue: CommandQueue,
    forw_kernel: Kernel,
    back_kernel: Kernel,

    light_field_geom_buf: Mem,
    volume_geom_buf: Mem,
    plane_to_geom_buf: Mem,
    geom_to_plane_buf: Mem,
}

impl<'geom, 'lfg, F: 'geom + 'lfg + Float + FromPrimitive + ToPrimitive + BaseFloat> 
LightVolumeTracer<'geom, 'lfg, F> {
    pub fn new(volume: &'geom LightVolume<F>,
               light_field: &'lfg LightFieldGeometry<F>,
               volume_isometry: &Isometry<F>,
               light_field_isometry: &Isometry<F>,
               queue: CommandQueue) -> Result<Self, Error> {
        // collect opencl sources
        let sources = match &light_field.plane.basis {
            &AngularBasis::Pillbox => [
                ImageGeometry::<F>::header(),
                Optics::<F>::header(),
                LightVolume::<F>::header(),
                include_str!("../cl/volume_trace_pillbox_f32.opencl"),
            ],
            &AngularBasis::Dirac => [
                ImageGeometry::<F>::header(),
                Optics::<F>::header(),
                LightVolume::<F>::header(),
                include_str!("../cl/volume_trace_dirac_f32.opencl"),
            ],
        };

        // compute isometries between volume and light field
        let plane_to_geom = volume_isometry.inverse_transformation().prepend_transformation(&light_field_isometry);
        let geom_to_plane = plane_to_geom.inverse_transformation();

        // compile opencl code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let unbuilt = try!(Program::new_from_source(context.clone(), &sources));
        let program = try!(unbuilt.build(&[device]));

        // create kernels
        let forw_kernel = try!(program.create_kernel("trace_forw"));
        let back_kernel = try!(program.create_kernel("trace_back"));

        // geometry/constant buffers
        let light_field_geom_buf = try!(light_field.geom.as_cl_buffer(&queue));
        let volume_geom_buf = try!(volume.as_cl_buffer(&queue));
        let plane_to_geom_buf = try!(plane_to_geom.as_cl_buffer(&queue));
        let geom_to_plane_buf = try!(geom_to_plane.as_cl_buffer(&queue));

        Ok(LightVolumeTracer{
            volume: volume,
            light_field: light_field,

            plane_to_geom: plane_to_geom,
            geom_to_plane: geom_to_plane,

            queue: queue,
            forw_kernel: forw_kernel,
            back_kernel: back_kernel,

            light_field_geom_buf: light_field_geom_buf,
            volume_geom_buf: volume_geom_buf,
            plane_to_geom_buf: plane_to_geom_buf,
            geom_to_plane_buf: geom_to_plane_buf,
        })
    }

    pub fn forw(self: &mut Self,
                vol: &Mem, lf: &mut Mem,
                ia: usize, wait_for: &[Event]) -> Result<Event, Error> {
        unimplemented!()
    }

    pub fn back(self: &mut Self,
                lf: &Mem, vol: &mut Mem,
                ia: usize, wait_for: &[Event]) -> Result<Event, Error> {
        unimplemented!()
    }
}

