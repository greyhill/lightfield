extern crate num;
extern crate proust;

use self::proust::*;
use image_geom::*;
use self::num::{FromPrimitive, Float};
use ellipsoid::*;
use optics::*;
use light_volume::*;
use cl_traits::*;

/// Renderer for phantoms
pub struct PhantomRenderer<F: Float> {
    pub geom: LightVolume<F>,

    geom_buf: Mem,

    render_ellipsoid: Kernel,

    queue: CommandQueue,
}

impl<F: Float> ClHeader for PhantomRenderer<F> {
    fn header() -> &'static str {
        include_str!("../cl/phantom_f32.opencl")
    }
}

impl<F: Float + FromPrimitive> PhantomRenderer<F> {
    pub fn new(geom: LightVolume<F>,
               queue: CommandQueue) -> Result<Self, Error> {
        let sources = &[
            ImageGeometry::<F>::header(),
            Optics::<F>::header(),
            LightVolume::<F>::header(),
            Ellipsoid::<F>::header(),
            Self::header(),
        ];

        // compile opencl code
        let context = try!(queue.context());
        let device = try!(queue.device());
        let unbuilt = try!(Program::new_from_source(context.clone(), sources));
        let program = try!(unbuilt.build(&[device]));

        // get kernels
        let render_ellipsoid = try!(program.create_kernel("render_ellipsoid"));

        // geometry buffer
        let geom_buf = try!(geom.as_cl_buffer(&queue));

        Ok(PhantomRenderer{
            geom: geom,
            geom_buf: geom_buf,
            render_ellipsoid: render_ellipsoid,
            queue: queue
        })
    }

    pub fn render_ellipsoid(self: &mut Self,
                            vol: &mut Mem,
                            ellipsoid: &Ellipsoid<F>,
                            wait_for: &[Event]) -> Result<Event, Error> {
        // create ellipsoid info buffer
        let ellipsoid_buf = try!(ellipsoid.as_cl_buffer(&self.queue));
        self.render_ellipsoids(1, 
                               &ellipsoid_buf,
                               vol,
                               wait_for)
    }

    pub fn render_ellipsoids(self: &mut Self,
                             num_ellipsoids: usize,
                             ellipsoids: &Mem,
                             vol: &mut Mem,
                             wait_for: &[Event]) -> Result<Event, Error> {
        // bind arguments
        try!(self.render_ellipsoid.bind(0, &self.geom_buf));
        try!(self.render_ellipsoid.bind(1, ellipsoids));
        try!(self.render_ellipsoid.bind_mut(2, vol));
        try!(self.render_ellipsoid.bind_scalar(3, &(num_ellipsoids as i32)));

        // run kernel
        let local_size = (32, 8, 1);
        let global_size = (self.geom.nx, self.geom.ny, self.geom.nz);

        self.queue.run_with_events(&mut self.render_ellipsoid,
                                   local_size,
                                   global_size,
                                   wait_for)
    }
}

#[test]
fn test_phantom_renderer() {
    use env::*;
    use geom::*;

    let env = Environment::new_easy().unwrap();
    let queue = &env.queues[0];

    let vg = LightVolume{
        nx: 100,
        ny: 200,
        nz: 300,
        dx: 1.0,
        dy: 2.0,
        dz: 3.0,
        offset_x: 1.0,
        offset_y: 2.0,
        offset_z: 3.0,
    };

    let mut v_buf = vg.zeros_buf(&queue).unwrap();
    let mut renderer = PhantomRenderer::new(vg.clone(), queue.clone()).unwrap();
    let ell = Ellipsoid::sphere(2f32, 3f32, 4f32, 20f32, 1f32);

    renderer.render_ellipsoid(&mut v_buf, &ell, &[]).unwrap().wait().unwrap();

    let mut v = vg.zeros();
    queue.read_buffer(&v_buf, &mut v).unwrap();

    let max_val = v.iter().fold(0f32, |l, &x| if x > l { x } else { l });
    assert_eq!(max_val, 1f32);
}

