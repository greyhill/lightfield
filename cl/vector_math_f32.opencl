// vim: filetype=opencl

kernel void VectorMath_set(int dimension,
                           global float* vec,
                           float val) {
    int idx = get_global_id(0);
    if(idx >= dimension) {
        return;
    }
    vec[idx] = val;
}

kernel void VectorMath_mix(int dimension,
        global float* x,
        global float* y,
        float ax,
        float ay,
        global float* out) {
    int idx = get_global_id(0);
    if(idx >= dimension) {
        return;
    }
    out[idx] = ax*x[idx] + ay*y[idx];
}

kernel void VectorMath_div(int dimension,
        global float* x,
        global float* y,
        global float* out) {
    int idx = get_global_id(0);
    if(idx >= dimension) {
        return;
    }
    float yi = y[idx];
    if(yi != 0.f) {
        out[idx] = x[idx] / y[idx];
    } else {
        out[idx] = 0.f;
    }
}

