// vim: filetype=opencl

struct ImageGeometry {
    int ns;
    int nt;
    float ds;
    float dt;
    float offset_s;
    float offset_t;
    float ws;
    float wt;
};
typedef constant struct ImageGeometry* ImageGeometry;

inline float ImageGeometry_is2s(ImageGeometry self, const int is) {
    return (is - self->ws)*self->ds;
}

inline float ImageGeometry_s2is(ImageGeometry self, float s) {
    return s/self->ds + self->ws + 0.5f;
}

inline float ImageGeometry_it2t(ImageGeometry self, const int it) {
    return (it - self->wt)*self->dt;
}

inline float ImageGeometry_t2it(ImageGeometry self, float t) {
    return t/self->dt + self->wt + 0.5f;
}

kernel void image_zero(
        ImageGeometry geom,
        global float* img) {
    int is = get_global_id(0);
    int it = get_global_id(1);

    if(is >= geom->ns || it >= geom->nt) {
        return;
    }

    img[is + geom->ns*it] = 0.f;
}

kernel void image_residual(
        ImageGeometry geom,
        global float* proj,
        float proj_scale,
        global float* measurements,
        global float* out) {
    int is = get_global_id(0);
    int it = get_global_id(1);

    if(is >= geom->ns || it >= geom->nt) {
        return;
    }

    int idx = is + geom->ns*it;

    out[idx] = proj_scale * proj[idx] - measurements[idx];
}

kernel void apply_mask(
        ImageGeometry geom,
        global const float* mask,
        global float* img) {
    const int is = get_global_id(0);
    const int it = get_global_id(1);

    if(is >= geom->ns || it >= geom->nt) {
        return;
    }

    // trick: masks with entries > 1 are clamped to 1; sometimes these
    // values are used to store extra information and the numbers [0..1]
    // are used for masking
    float mask_val = mask[is + geom->ns*it];
    mask_val = fmin(1.f, mask_val);

    img[is + geom->ns*it] *= mask_val;
}

kernel void apply_mask_to(
        ImageGeometry geom,
        global const float* mask,
        global const float* img,
        global float* out) {
    const int is = get_global_id(0);
    const int it = get_global_id(1);

    if(is >= geom->ns || it >= geom->nt) {
        return;
    }

    // trick: masks with entries > 1 are clamped to 1; sometimes these
    // values are used to store extra information and the numbers [0..1]
    // are used for masking
    float mask_val = mask[is + geom->ns*it];
    mask_val = fmin(1.f, mask_val);

    out[is + geom->ns*it] = img[is + geom->ns*it] * mask_val;
}


