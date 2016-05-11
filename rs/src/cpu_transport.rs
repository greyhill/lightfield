use light_field_geometry::*;
use angular_plane::*;
use stage::*;
use cpu_context::*;
use float_util::*;
use filter::*;

/// Multithreaded plane-to-plane transport
pub struct CpuTransport<'src, 'dst> {
    src: &'src LightFieldGeometry,
    dst: &'dst LightFieldGeometry,
}

impl<'src, 'dst> CpuTransport<'src, 'dst> {
    pub fn new(src: &'src LightFieldGeometry,
               dst: &'dst LightFieldGeometry) -> Self {
        CpuTransport{
            src: src,
            dst: dst,
        }
    }

    fn filter_angle_dirac(self: &Self,
                           ctx: &mut CpuContext,
                           iu: usize, iv: usize,
                           input: &[f32],
                           output: &mut [f32],
                           forw: bool) -> Result<(), ()> {
        let (p, q) = if forw {
            (self.src, self.dst)
        } else {
            (self.dst, self.src)
        };
        let angular = &p.angular_plane;

        let Rp = &p.optics_to_plane;
        let Rq = &q.optics_to_plane;
        let Rqp = Rq.invert().compose(Rp);

        let u = angular.geom.is2s(iu);
        let v = angular.geom.it2t(iv);

        let filter_x = match &angular.coordinate {
            &AngularCoordinate::Space => {
                let alpha_x = Rqp.ss() - Rp.ss()*Rqp.su()/Rp.su();
                let beta_x = Rqp.su()*(u - Rp.s())/Rp.su();

                let mut h_x = (angular.geom.ds / Rp.su()).abs();
                if !forw {
                    h_x /= q.pixel_volume();
                }
                h_x *= p.geom.ds;

                let t0x = p.geom.s2is((q.geom.ds / 2f32 - beta_x)/alpha_x);
                let t1x = p.geom.s2is((- q.geom.ds / 2f32 - beta_x)/alpha_x);

                Rect{
                    height: h_x,
                    i_scale: 1f32 / alpha_x,
                    t0: fmin(t0x, t1x),
                    t1: fmax(t0x, t1x),
                }
            },
            &AngularCoordinate::Angle => {
                let alpha_x = Rqp.ss() - Rp.us()*Rqp.su()/Rp.uu();
                let beta_x = Rqp.su()*(u - Rp.u())/Rp.uu();

                let mut h_x = (angular.geom.ds / Rp.uu()).abs();
                if !forw {
                    h_x /= q.pixel_volume();
                }
                h_x *= p.geom.ds;

                let t0x = p.geom.s2is((q.geom.ds / 2f32 - beta_x)/alpha_x);
                let t1x = p.geom.s2is((-q.geom.ds / 2f32 - beta_x)/alpha_x);

                Rect{
                    height: h_x,
                    i_scale: 1f32 / alpha_x,
                    t0: fmin(t0x, t1x),
                    t1: fmax(t0x, t1x),
                }
            },
        };
        let filter_y = match &angular.coordinate {
            &AngularCoordinate::Space => {
            },
            &AngularCoordinate::Angle => {
            },
        };
    }

    fn filter_angle_pillbox(self: &Self,
                           ctx: &mut CpuContext,
                           iu: usize, iv: usize,
                           input: &[f32],
                           output: &mut [f32],
                           forw: bool) -> Result<(), ()> {
        unimplemented!()
    }
}

impl<'src, 'dst> Stage<CpuContext> for CpuTransport<'src, 'dst> {
    fn forw_angle(self: &Self,
                  context: &mut CpuContext,
                  iu: usize, iv: usize,
                  input: &[f32],
                  output: &mut [f32]) -> Result<(), ()> {
        match &self.src.angular_plane.basis {
            &AngularBasis::Dirac => {
                self.filter_angle_dirac(context, iu, iv, input, output, true)
            },
            &AngularBasis::Pillbox => {
                self.filter_angle_pillbox(context, iu, iv, input, output, true)
            },
        }
    }

    fn back_angle(self: &Self,
                  context: &mut CpuContext,
                  iu: usize, iv: usize,
                  input: &[f32],
                  output: &mut [f32]) -> Result<(), ()> {
        match &self.src.angular_plane.basis {
            &AngularBasis::Dirac => {
                self.filter_angle_dirac(context, iu, iv, input, output, false)
            },
            &AngularBasis::Pillbox => {
                self.filter_angle_pillbox(context, iu, iv, input, output, false)
            },
        }
    }
}

