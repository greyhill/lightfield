#include "lightfield/lightfield.h"

static float LFLixel_volume_dirac_spatial(
        const struct LFPlaneGeometry* plane,
        const struct LFAngularPlane* angular_plane,
        const struct LFOptics* to_plane_x,
        const struct LFOptics* to_plane_y) {
    const float dx = fabs(angular_plane->du / to_plane_x->pa) * plane->ds;
    const float dy = fabs(angular_plane->dv / to_plane_y->pa) * plane->dt;
    return dx*dy;
}

static float LFLixel_volume_dirac_angular(
        const struct LFPlaneGeometry* plane,
        const struct LFAngularPlane* angular_plane,
        const struct LFOptics* to_plane_x,
        const struct LFOptics* to_plane_y) {
    const float dx = fabs(angular_plane->du / to_plane_x->aa) * plane->ds;
    const float dy = fabs(angular_plane->dv / to_plane_y->aa) * plane->dt;
    return dx*dy;
}

static float LFLixel_volume_box_spatial(
        const struct LFPlaneGeometry* plane,
        const struct LFAngularPlane* angular_plane,
        const struct LFOptics* to_plane_x,
        const struct LFOptics* to_plane_y) {
    const float Mx = fmax(angular_plane->du / 2.f / fabs(to_plane_x->pa),
                          plane->ds / 2.f * fabs(to_plane_x->pp / to_plane_x->pa));
    const float hx = fmin(plane->ds, angular_plane->du / fabs(to_plane_x->pp));
    const float My = fmax(angular_plane->dv / 2.f / fabs(to_plane_y->pa),
                          plane->dt / 2.f * fabs(to_plane_y->pp / to_plane_y->pa));
    const float hy = fmin(plane->dt, angular_plane->dv / fabs(to_plane_y->pp));
    return 4.f * Mx * hx * My * hy;
}

static float LFLixel_volume_box_angular(
        const struct LFPlaneGeometry* plane,
        const struct LFAngularPlane* angular_plane,
        const struct LFOptics* to_plane_x,
        const struct LFOptics* to_plane_y) {
    const float Mx = fmax(angular_plane->du / 2.f / fabs(to_plane_x->aa),
                          plane->ds / 2.f * fabs(to_plane_x->ap / to_plane_x->aa));
    const float hx = fmin(plane->ds, angular_plane->du / fabs(to_plane_x->ap));
    const float My = fmax(angular_plane->dv / 2.f / fabs(to_plane_y->aa),
                          plane->dt / 2.f * fabs(to_plane_y->ap / to_plane_y->aa));
    const float hy = fmin(plane->dt, angular_plane->dv / fabs(to_plane_y->ap));
    return 4.f * Mx * hx * My * hy;
}

float LFLixel_volume(
        const struct LFPlaneGeometry* plane,
        const struct LFAngularPlane* angular_plane,
        const struct LFOptics* to_plane_x,
        const struct LFOptics* to_plane_y) {
    switch(angular_plane->type | angular_plane->coordinate) {
        case LF_PLANE_DIRAC | LF_PLANE_SPATIAL: 
            return LFLixel_volume_dirac_spatial(plane, angular_plane, to_plane_x, to_plane_y);

        case LF_PLANE_DIRAC | LF_PLANE_ANGULAR:
            return LFLixel_volume_dirac_angular(plane, angular_plane, to_plane_x, to_plane_y);

        case LF_PLANE_BOX | LF_PLANE_SPATIAL:
            return LFLixel_volume_box_spatial(plane, angular_plane, to_plane_x, to_plane_y);

        case LF_PLANE_BOX | LF_PLANE_ANGULAR:
            return LFLixel_volume_box_angular(plane, angular_plane, to_plane_x, to_plane_y);

        default:
            return NAN;
    }
}

