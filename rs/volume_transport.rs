extern crate num;
extern crate proust;
use self::num::{Float, FromPrimitive};
use self::proust::*;

use light_volume::*;
use light_field_geom::*;
use optics::*;
use angular_plane::*;
use image_geom::*;
use cl_traits::*;
use spline_kernel::*;

use std::cmp::max;
use std::mem::size_of;

/// Transport for `LightVolume` objects
pub struct VolumeTransport<F: Float> {
    pub geom: LightVolume<F>,
    pub dst: LightFieldGeometry<F>,

    queue: CommandQueue,
    forw_t_kernel: Kernel,
    forw_s_kernel: Kernel,

    tmp: Mem,                   // half-filtered volume
    volume_geom: Mem,           // LightVolumeGeometry
    dst_geom: Mem,              // ImageGeometry
    slice_geom: Mem,            // ImageGeometry

    slice_to_dst: Mem,          // [Optics]*nz
    dst_to_slice: Mem,          // [Optics]*nz
    forw_spline_kernels_s: Mem, // [SplineKernel]*nz*na
    forw_spline_kernels_t: Mem, // [SplineKernel]*nz*na
    back_spline_kernels_s: Mem, // [SplineKernel]*nz*na
    back_spline_kernels_t: Mem, // [SplineKernel]*nz*na
}

impl<F> VolumeTransport<F>
where F: Float + FromPrimitive {
    /// Create a new `VolumeTransport`
    ///
    /// The `dst` light field geometry is placed at the center (`z=0`) of the 
    /// volume.
    pub fn new(src: LightVolume<F>,
               dst: LightFieldGeometry<F>,
               to_dst: Optics<F>,
               queue: CommandQueue) -> Result<Self, Error> {
        // collect opencl sources
        let sources = match &dst.plane.basis {
            &AngularBasis::Pillbox => {
                [
                    ImageGeometry::<F>::header(),
                    Optics::<F>::header(),
                    LightVolume::<F>::header(),
                    SplineKernel::<F>::header(),
                    include_str!("../cl/transport_pillbox_f32.opencl"),
                    include_str!("../cl/volume_transport_pillbox_f32.opencl"),
                ]
            },
            &AngularBasis::Dirac => {
                [
                    ImageGeometry::<F>::header(),
                    Optics::<F>::header(),
                    LightVolume::<F>::header(),
                    SplineKernel::<F>::header(),
                    include_str!("../cl/transport_dirac_f32.opencl"),
                    include_str!("../cl/volume_transport_dirac_f32.opencl"),
                ]
            },
        };

        // compile opencl code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let unbuilt = try!(Program::new_from_source(context.clone(), &sources));
        let program = try!(unbuilt.build(&[device]));

        // get opencl kernels
        let forw_t_kernel = try!(program.create_kernel("volume_forw_t"));
        let forw_s_kernel = try!(program.create_kernel("volume_forw_s"));

        // size of temporary buffers
        let tmp_nx = max(src.nx, dst.geom.ns);
        let tmp_ny = max(src.ny, dst.geom.nt);

        // global buffers
        let tmp = try!(queue.create_buffer(size_of::<F>() * tmp_nx * tmp_ny));
        let volume_geom = try!(src.as_cl_buffer(&queue));
        let dst_geom = try!(dst.geom.as_cl_buffer(&queue));
        let slice_geom = try!(src.transaxial_image_geometry().as_cl_buffer(&queue));

        // slice buffers
        //
        // we precompute the footprints for each angle and slice.  this takes
        // 4 or 6 floats per direction (s and t) and slice (nz) for each angle.
        // in total, this takes
        //      4 * (4 | 6) * Na * Nz * 2
        // bytes, which isn't much for reasonable problem sizes
        let mut forw_spline_kernels_s_buf: Vec<u8> = Vec::new();
        let mut forw_spline_kernels_t_buf: Vec<u8> = Vec::new();
        let mut back_spline_kernels_s_buf: Vec<u8> = Vec::new();
        let mut back_spline_kernels_t_buf: Vec<u8> = Vec::new();
        for iz in 0 .. src.nz {
            let slice_lfg = src.slice_light_field_geometry(iz, dst.plane.clone(), to_dst.clone());
            for ia in 0 .. dst.plane.s.len() {
                let (forw_s, forw_t) = slice_lfg.transport_to(&dst, ia);
                let (back_s, back_t) = dst.transport_to(&slice_lfg, ia);

                forw_s.as_cl_bytes(&mut forw_spline_kernels_s_buf);
                forw_t.as_cl_bytes(&mut forw_spline_kernels_t_buf);
                back_s.as_cl_bytes(&mut back_spline_kernels_s_buf);
                back_t.as_cl_bytes(&mut back_spline_kernels_t_buf);
            }
        }

        // we also precompute the optical transformations between each slice
        // and the destination, and from the destination to each slice
        let mut slice_to_dst_buf: Vec<u8> = Vec::new();
        let mut dst_to_slice_buf: Vec<u8> = Vec::new();
        for iz in 0 .. src.nz {
            let slice_lfg = src.slice_light_field_geometry(iz, dst.plane.clone(), to_dst.clone());
            let src2dst = slice_lfg.optics_to(&dst);
            let dst2src = src2dst.invert();

            src2dst.as_cl_bytes(&mut slice_to_dst_buf);
            dst2src.as_cl_bytes(&mut dst_to_slice_buf);
        }

        // load precomputed values onto the GPU
        let slice_to_dst = try!(queue.create_buffer_from_slice(&slice_to_dst_buf));
        let dst_to_slice = try!(queue.create_buffer_from_slice(&dst_to_slice_buf));
        let forw_spline_kernels_s = try!(queue.create_buffer_from_slice(&forw_spline_kernels_s_buf));
        let forw_spline_kernels_t = try!(queue.create_buffer_from_slice(&forw_spline_kernels_t_buf));
        let back_spline_kernels_s = try!(queue.create_buffer_from_slice(&back_spline_kernels_s_buf));
        let back_spline_kernels_t = try!(queue.create_buffer_from_slice(&back_spline_kernels_t_buf));

        Ok(VolumeTransport{
            geom: src,
            dst: dst,

            queue: queue,
            forw_t_kernel: forw_t_kernel,
            forw_s_kernel: forw_s_kernel,

            tmp: tmp,
            volume_geom: volume_geom,
            dst_geom: dst_geom,
            slice_geom: slice_geom,

            slice_to_dst: slice_to_dst,
            dst_to_slice: dst_to_slice,
            forw_spline_kernels_s: forw_spline_kernels_s,
            forw_spline_kernels_t: forw_spline_kernels_t,
            back_spline_kernels_s: back_spline_kernels_s,
            back_spline_kernels_t: back_spline_kernels_t,
        })
    }
}

#[test]
fn test_volume_dirac() {
    use env::*;
    use lens::*;

    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

   let lens = Lens{
        center_s: 1f32,
        center_t: -1.5f32,
        radius_s: 20f32,
        radius_t: 15f32,
        focal_length_s: 30f32,
        focal_length_t: 35f32,
    };
    let plane = lens.as_angular_plane(AngularBasis::Dirac, 20);

    let vg = LightVolume{
        nx: 100,
        ny: 200,
        nz: 300,
        dx: 3.0,
        dy: 2.0,
        dz: 1.0,
        offset_x: -1.0,
        offset_y: 2.0,
        offset_z: -3.0,
    };

    let dst_geom = ImageGeometry{
        ns: 1024,
        nt: 2048,
        ds: 5e-2,
        dt: 3e-2,
        offset_s: -4.0,
        offset_t: 2.1,
    };

    let dst = LightFieldGeometry{
        geom: dst_geom,
        plane: plane.clone(),
        to_plane: Optics::translation(&40f32),
    };

    let to_dst = lens.optics().then(&Optics::translation(&500f32)).invert();

    let xport = VolumeTransport::new(vg, dst, to_dst, queue.clone()).unwrap();
}

