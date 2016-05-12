#include "lightfield/lightfield.h"

bool LFTransport_init(struct LFTransport* x) {
    x->angular_plane = NULL;
    x->src_plane = NULL;
    x->src_to_root_s = NULL;
    x->src_to_root_t = NULL;
    x->dst_plane = NULL;
    x->dst_to_root_s = NULL;
    x->dst_to_root_t = NULL;
    x->scale = NAN;
    LFOptics_identity(&x->src_to_dst_s);
    LFOptics_identity(&x->src_to_dst_t);
    return true;
}

bool LFTransport_del(struct LFTransport* x) {
    x->angular_plane = NULL;
    x->src_plane = NULL;
    x->src_to_root_s = NULL;
    x->src_to_root_t = NULL;
    x->dst_plane = NULL;
    x->dst_to_root_s = NULL;
    x->dst_to_root_t = NULL;
    x->scale = NAN;
    LFOptics_identity(&x->src_to_dst_s);
    LFOptics_identity(&x->src_to_dst_t);
    return true;
}

size_t LFTransport_tmp_size(const struct LFTransport* x) {
    return x->dst_plane->nt * x->src_plane->ns;
}

bool LFTransport_setup(struct LFTransport* x,
        const struct LFAngularPlane* angular_plane,
        const struct LFPlaneGeometry* src_plane,
        const struct LFOptics* src_to_root_s,
        const struct LFOptics* src_to_root_t,
        const struct LFPlaneGeometry* dst_plane,
        const struct LFOptics* dst_to_root_s,
        const struct LFOptics* dst_to_root_t,
        const float scale) {
    x->angular_plane = angular_plane;
    x->src_plane = src_plane;
    x->src_to_root_s = src_to_root_s;
    x->src_to_root_t = src_to_root_t;
    x->dst_plane = dst_plane;
    x->dst_to_root_s = dst_to_root_s;
    x->dst_to_root_t = dst_to_root_t;
    x->scale = scale;

    // src -> dst = (src -> root) * inverse(dst -> root)
    // compute inverse of dst->root first and store,
    // then compose with src -> root in-place
    LFOptics_invert(x->dst_to_root_s, &x->src_to_dst_s);
    LFOptics_compose(x->src_to_root_s, &x->src_to_dst_s, &x->src_to_dst_s);
    LFOptics_invert(x->dst_to_root_t, &x->src_to_dst_t);
    LFOptics_compose(x->src_to_root_s, &x->src_to_dst_t, &x->src_to_dst_t);

    return true;
}

bool LFTransport_compute(struct LFTransport* x,
        size_t i_view,
        cl_mem src, cl_mem dst, cl_mem tmp) {
}

