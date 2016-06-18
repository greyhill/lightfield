// vim: filetype=opencl

kernel void FistaVolumeSolver_update(
        LightVolume geom,
        PotentialFunction sparsifying,
        global float* x,
        global float* denom,
        global float* data_gradient,
        float min_val,
        float max_val,
        global float* m,
        float t0,
        float t1,
        global float* mask3) {
    const int ix = get_global_id(0);
    const int iy = get_global_id(1);
    const int iz = get_global_id(2);

    if(ix >= geom->nx || iy >= geom->ny || iz >= geom->nz) {
        return;
    }

    const int idx = ix + geom->nx*(iy + geom->ny*iz);

    const float xi = x[idx];
    const float gi = data_gradient[idx];
    const float di = denom[idx];
    const float mi = m[idx];
    float m3i = mask3[idx];

    if(m3i > 0.f) {
        m3i = 0.f;
    } else {
        m3i = 1.f;
    }

    float new_val = xi - gi / di;

    if(sparsifying != NULL) {
        new_val = PotentialFunction_shrink(sparsifying, di, new_val);
    }

    if(new_val < min_val) {
        new_val = min_val;
    } else if(new_val > max_val) {
        new_val = max_val;
    }

    x[idx] = m3i*(new_val + (t0 - 1.f)/t1*(new_val - mi));
    m[idx] = m3i*new_val;
}

