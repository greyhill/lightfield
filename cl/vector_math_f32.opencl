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
    vec[idx] = ax*x[idx] + ay*y[idx];
}

