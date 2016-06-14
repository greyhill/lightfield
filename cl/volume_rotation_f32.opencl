// vim: filetype=opencl

kernel void rotate_filter_z(
        LightVolume geom,
        global struct TrapezoidSplineKernel* kernels,
        global const float* input,
        global float* output) {
    const int ix = get_global_id(0);
    const int iy = get_global_id(1);
    const int iz = get_global_id(2);

    if(ix >= geom->nx || iy >= geom->ny || iz >= geom->nz) {
        return;
    }

    const float z = LightVolume_iz2z(geom, iz);

    global struct TrapezoidSplineKernel* my_kernel = kernels + ix + geom->nx*iy;

    const float src_fz0 = LightVolume_z2iz(geom, z + my_kernel->tau0);
    const float src_fz1 = LightVolume_z2iz(geom, z + my_kernel->tau3);
    const int src_iz0 = fmax(0.f, fmin(src_fz0, geom->nz));
    const int src_iz1 = fmax(0.f, fmin(ceil(src_fz1), geom->nz));

    float accum = 0.f;
    for(int src_iz = src_iz0; src_iz < src_iz1; ++src_iz) {
        const float src_z = LightVolume_iz2z(geom, src_iz);
        accum += TrapezoidSplineKernel_sample(my_kernel, z, src_z) * input[ix + geom->nx*(iy + geom->ny*src_iz)];
    }

    output[ix + geom->nx*(iy + geom->ny*iz)] = accum;
}

kernel void rotate_filter_y(
        LightVolume geom,
        global struct TrapezoidSplineKernel* kernels,
        global const float* input,
        global float* output) {
    const int ix = get_global_id(0);
    const int iy = get_global_id(1);
    const int iz = get_global_id(2);

    if(ix >= geom->nx || iy >= geom->ny || iz >= geom->nz) {
        return;
    }

    const float y = LightVolume_iy2y(geom, iy);

    global struct TrapezoidSplineKernel* my_kernel = kernels + ix + geom->nx*iz;

    const float src_fy0 = LightVolume_y2iy(geom, y + my_kernel->tau0);
    const float src_fy1 = LightVolume_y2iy(geom, y + my_kernel->tau3);
    const int src_iy0 = fmax(0.f, fmin(src_fy0, geom->ny));
    const int src_iy1 = fmax(0.f, fmin(ceil(src_fy1), geom->ny));

    float accum = 0.f;
    for(int src_iy = src_iy0; src_iy < src_iy1; ++src_iy) {
        const float src_y0 = LightVolume_iy2y(geom, src_iy) - fabs(geom->dy)/2.f;
        const float src_y1 = src_y0 + fabs(geom->dy)/2.f;
        accum += TrapezoidSplineKernel_integrate(my_kernel, y, src_y0, src_y1) * input[ix + geom->nx*(src_iy + geom->ny*iz)];
    }

    output[ix + geom->nx*(iy + geom->ny*iz)] = accum;
}

kernel void rotate_filter_x(
        LightVolume geom,
        global struct TrapezoidSplineKernel* kernels,
        global const float* input,
        global float* output) {
    const int ix = get_global_id(0);
    const int iy = get_global_id(1);
    const int iz = get_global_id(2);

    if(ix >= geom->nx || iy >= geom->ny || iz >= geom->nz) {
        return;
    }

    const float x = LightVolume_ix2x(geom, ix);

    global struct TrapezoidSplineKernel* my_kernel = kernels + iy + geom->nx*iz;

    const float src_fx0 = LightVolume_x2ix(geom, x + my_kernel->tau0);
    const float src_fx1 = LightVolume_x2ix(geom, x + my_kernel->tau3);
    const int src_ix0 = fmax(0.f, fmin(src_fx0, geom->nx));
    const int src_ix1 = fmax(0.f, fmin(ceil(src_fx1), geom->nx));

    float accum = 0.f;
    for(int src_ix = src_ix0; src_ix < src_ix1; ++src_ix) {
        const float src_x0 = LightVolume_ix2x(geom, src_ix) - fabs(geom->dx)/2.f;
        const float src_x1 = src_x0 + fabs(geom->dx)/2.f;
        accum += TrapezoidSplineKernel_integrate(my_kernel, x, src_x0, src_x1) * input[src_ix + geom->nx*(iy + geom->ny*iz)];
    }

    output[ix + geom->nx*(iy + geom->ny*iz)] = accum;
}

