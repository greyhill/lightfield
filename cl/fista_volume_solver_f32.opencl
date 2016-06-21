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
        global float* mask3,
        PotentialFunction edge_preserving,
        global float* x_off) {
    const int ix = get_global_id(0);
    const int iy = get_global_id(1);
    const int iz = get_global_id(2);

    if(ix >= geom->nx || iy >= geom->ny || iz >= geom->nz) {
        return;
    }

    const int idx = ix + geom->nx*(iy + geom->ny*iz);

    const float xi = x_off[idx];
    float gi = data_gradient[idx];
    float di = denom[idx];
    const float mi = m[idx];
    float m3i = mask3[idx];

    if(m3i > 0.f) {
        m3i = 0.f;
    } else {
        m3i = 1.f;
    }

    if(edge_preserving != NULL) {
        for(int iiz=max(iz-1, 0); iiz<min(iz+2, geom->nz); ++iiz) {
            for(int iiy=max(iy-1, 0); iiy<min(iy+2, geom->ny); ++iiy) {
                for(int iix=max(ix-1, 0); iix<min(ix+2, geom->nx); ++iix) {
                    const int iidx = iix + geom->nx*(iiy + geom->ny*iiz);
                    const float xii = x_off[iidx];
                    const float h = PotentialFunction_huber(edge_preserving, xi - xii);
                    di += h;
                    gi += h*(xi - xii);
                }
            }
        }
    }

    float new_val = xi - gi / di;
    if(di == 0.f) {
        new_val = 0.f;
    }

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

