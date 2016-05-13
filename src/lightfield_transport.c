#include "lightfield/lightfield.h"

static bool LFTransport_compute_dirac(struct LFTransport* x,
        size_t i_view, cl_mem src, cl_mem dst, cl_mem tmp);
static bool LFTransport_compute_pillbox(struct LFTransport* x,
        size_t i_view, cl_mem src, cl_mem dst, cl_mem tmp);

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
        size_t i_view, cl_mem src, cl_mem dst, cl_mem tmp) {
    LF_ERROR_START;
    if(x->angular_plane->type == LF_PLANE_DIRAC) {
        LF_TRY(LFTransport_compute_dirac(x, i_view, src, dst, tmp));
    } else if(x->angular_plane->type == LF_PLANE_BOX) {
        LF_TRY(LFTransport_compute_pillbox(x, i_view, src, dst, tmp));
    } else {
        LF_TRY(false);
    }
    LF_ERROR_BLOCK;
    return ok;
}

bool LFTransport_compute_dirac(struct LFTransport* x,
        size_t i_view, cl_mem src, cl_mem dst, cl_mem tmp) {
    LF_ERROR_START;
    cl_command_queue q = LFCL_get_queue();
    LF_TRY(q != NULL);

    // alpha_, beta_, h_ for s and t
    float a_s, b_s, h_s;
    float a_t, b_t, h_t;

    // angular plane positions
    const float u = x->angular_plane->u_points[i_view];
    const float v = x->angular_plane->v_points[i_view];

    // convenience references
    const struct LFOptics* Rqps = &x->src_to_dst_s;
    const struct LFOptics* Rps = x->src_to_root_s;
    const struct LFOptics* Rqs = x->dst_to_root_s;

    const struct LFOptics* Rqpt = &x->src_to_dst_t;
    const struct LFOptics* Rpt = x->src_to_root_t;
    const struct LFOptics* Rqt = x->dst_to_root_t;

    // compute parameters a, b, h
    // See Table 1
    if(x->angular_plane->coordinate == LF_PLANE_SPATIAL) {
        a_s = Rqps->pp - Rps->pp * Rqps->pa / Rps->pa;
        b_s = Rqps->pa*(u - Rps->cp)/Rps->pa;
        h_s = fabsf(x->angular_plane->du / Rqs->pa);

        a_t = Rqpt->pp - Rpt->pp * Rqpt->pa / Rpt->pa;
        b_t = Rqpt->pa*(v - Rpt->cp)/Rpt->pa;
        h_t = fabsf(x->angular_plane->dv / Rqt->pa);
    } else if(x->angular_plane->coordinate == LF_PLANE_ANGULAR) {
        a_s = Rqps->pp - Rps->ap * Rqps->pa / Rps->aa;
        b_s = Rqps->pa*(u - Rps->ca)/Rps->aa;
        h_s = fabsf(x->angular_plane->du / Rqs->aa);

        a_t = Rqpt->pp - Rpt->ap * Rqpt->pa / Rpt->aa;
        b_t = Rqpt->pa*(u - Rpt->ca)/Rpt->aa;
        h_t = fabsf(x->angular_plane->dv / Rqt->aa);
    } else {
        LF_TRY(false);
    }

    // Compute parameters
    // See Table 1
    const int src_ns = x->src_plane->ns;
    const int src_s0 = 0;
    const int src_s1 = src_ns;
    const int src_nt = x->src_plane->nt;
    const int src_t0 = 0;
    const int src_t1 = src_nt;

    const int dst_ns = x->dst_plane->ns;
    const int dst_s0 = 0;
    const int dst_s1 = dst_ns;
    const int dst_nt = x->dst_plane->nt;
    const int dst_t0 = 0;
    const int dst_t1 = dst_nt;

    const float coord_scale_t;
    const float tau0_t;
    const float tau1_t;
    const float scale_t;

    const float coord_scale_s;
    const float tau0_s;
    const float tau1_s;
    const float scale_s;

    const size_t global_size_t[3];
    const size_t local_size_t[3];

    const size_t global_size_s[3];
    const size_t local_size_s[3];

    // Filter t
    cl_kernel filter_t = LFCL_dirac_transport_filter_t();
    LF_TRY(filter_t != NULL);
    LF_CL_ARG(filter_t, 0, src_ns);
    LF_CL_ARG(filter_t, 1, src_s0);
    LF_CL_ARG(filter_t, 2, src_s1);
    LF_CL_ARG(filter_t, 3, src_nt);
    LF_CL_ARG(filter_t, 4, src_t0);
    LF_CL_ARG(filter_t, 5, src_t1);
    LF_CL_ARG(filter_t, 6, dst_ns);
    LF_CL_ARG(filter_t, 7, dst_s0);
    LF_CL_ARG(filter_t, 8, dst_s1);
    LF_CL_ARG(filter_t, 9, dst_nt);
    LF_CL_ARG(filter_t, 10, dst_t0);
    LF_CL_ARG(filter_t, 11, dst_t1);
    LF_CL_ARG(filter_t, 12, coord_scale_t);
    LF_CL_ARG(filter_t, 13, tau0_t);
    LF_CL_ARG(filter_t, 14, tau1_t);
    LF_CL_ARG(filter_t, 15, scale_t);
    LF_CL_ARG(filter_t, 16, src);
    LF_CL_ARG(filter_t, 17, tmp);

    cl_err = clEnqueueNDRangeKernel(q, filter_t, 2,
            NULL, global_size_t, local_size_t,
            0, NULL, NULL);
    LF_CHECK_CL;

    // Filter s
    cl_kernel filter_s = LFCL_dirac_transport_filter_s();
    LF_TRY(filter_s != NULL);

    LF_TRY(filter_t != NULL);
    LF_CL_ARG(filter_s, 0, src_ns);
    LF_CL_ARG(filter_s, 1, src_s0);
    LF_CL_ARG(filter_s, 2, src_s1);
    LF_CL_ARG(filter_s, 3, src_nt);
    LF_CL_ARG(filter_s, 4, src_t0);
    LF_CL_ARG(filter_s, 5, src_t1);
    LF_CL_ARG(filter_s, 6, dst_ns);
    LF_CL_ARG(filter_s, 7, dst_s0);
    LF_CL_ARG(filter_s, 8, dst_s1);
    LF_CL_ARG(filter_s, 9, dst_nt);
    LF_CL_ARG(filter_s, 10, dst_t0);
    LF_CL_ARG(filter_s, 11, dst_t1);
    LF_CL_ARG(filter_s, 12, coord_scale_s);
    LF_CL_ARG(filter_s, 13, tau0_s);
    LF_CL_ARG(filter_s, 14, tau1_s);
    LF_CL_ARG(filter_s, 15, scale_s);
    LF_CL_ARG(filter_s, 16, tmp);
    LF_CL_ARG(filter_s, 17, dst);

    cl_err = clEnqueueNDRangeKernel(q, filter_s, 2,
            NULL, global_size_s, local_size_s,
            0, NULL, NULL);
    LF_CHECK_CL;

    LF_ERROR_BLOCK;
    return false;
}

bool LFTransport_compute_pillbox(struct LFTransport* x,
        size_t i_view, cl_mem src, cl_mem dst, cl_mem tmp) {
    (void)x;
    (void)i_view;
    (void)src;
    (void)dst;
    (void)tmp;
    return false;
}

