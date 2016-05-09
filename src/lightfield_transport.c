#include "lightfield/lightfield.h"

void LFTransport_init(struct LFTransport* x) {
    x->src_plane = NULL;
    x->dst_plane = NULL;
    x->angular_plane = NULL;
    x->src2plane_x = NULL;
    x->src2plane_y = NULL;
    x->dst2plane_x = NULL;
    x->dst2plane_y = NULL;
    LFOptics_identity(&x->src2dst_x);
    LFOptics_identity(&x->src2dst_y);
    LFOptics_identity(&x->dst2src_x);
    LFOptics_identity(&x->dst2src_y);
}

void LFTransport_setup(struct LFTransport* x,
        const struct LFPlaneGeometry* src_plane,
        const struct LFPlaneGeometry* dst_plane,
        const struct LFAngularPlane* angular_plane,
        const struct LFOptics* src2plane_x,
        const struct LFOptics* src2plane_y,
        const struct LFOptics* dst2plane_x,
        const struct LFOptics* dst2plane_y) {
    x->src_plane = src_plane;
    x->dst_plane = dst_plane;
    x->angular_plane = angular_plane;
    x->src2plane_x = src2plane_x;
    x->src2plane_y = src2plane_y;
    x->dst2plane_x = dst2plane_x;
    x->dst2plane_y = dst2plane_y;

    // src2dst: (dst2plane)^{-1} src2plane
    LFOptics_invert(x->dst2plane_x, &x->src2dst_x);
    LFOptics_compose(&x->src2dst_x, x->src2plane_x, &x->src2dst_x);
    LFOptics_invert(x->dst2plane_y, &x->src2dst_y);
    LFOptics_compose(&x->src2dst_y, x->src2plane_y, &x->src2dst_y);

    // dst2src: (src2plane)^{-1} dst2plane
    LFOptics_invert(x->src2plane_x, &x->dst2src_x);
    LFOptics_compose(&x->dst2src_x, x->dst2plane_x, &x->dst2src_x);
    LFOptics_invert(x->src2plane_y, &x->dst2src_y);
    LFOptics_compose(&x->dst2src_y, x->dst2plane_y, &x->dst2src_y);
}

