// vim: filetype=opencl

kernel void render_ellipsoid(
        LightVolume geom,
        Ellipsoid ell,
        global float* ph) {
    int ix = get_global_id(0);
    int iy = get_global_id(1);
    int iz = get_global_id(2);

    if(ix >= geom->nx || iy >= geom->ny || iz >= geom->nz) {
        return;
    }

    const int num_samples_per_dim = 10;
    float accum = 0.f;

    const float x = LightVolume_ix2x(geom, ix) - geom->dx/2.f;
    const float y = LightVolume_iy2y(geom, iy) - geom->dy/2.f;
    const float z = LightVolume_iz2z(geom, iz) - geom->dz/2.f;

    for(int iix=0; iix<num_samples_per_dim; ++iix) {
        const float xx = x + geom->dx*iix/(num_samples_per_dim - 1.f);
        for(int iiy=0; iiy<num_samples_per_dim; ++iiy) {
            const float yy = y + geom->dy*iiy/(num_samples_per_dim - 1.f);
            for(int iiz=0; iiz<num_samples_per_dim; ++iiy) {
                const float zz = z + geom->dz*iiz/(num_samples_per_dim - 1.f);
                accum += Ellipsoid_eval(ell, xx, yy, zz);
            }
        }
    }

    accum /= num_samples_per_dim*num_samples_per_dim*num_samples_per_dim;

    ph[ix + geom->nx*(iy + geom->ny*iz)] += accum;
}

