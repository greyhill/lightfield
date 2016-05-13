// vim: filetype=opencl

kernel void filter_t(
        int src_ns, int src_s0, int src_s1,
        int src_nt, int src_t0, int src_t1,

        int dst_ns, int dst_s0, int dst_s1,
        int dst_nt, int dst_t0, int dst_t1,

        float coord_scale_t,
        float tau0_t,
        float tau1_t,
        float scale_t,

        global const float* input,
        global float* tmp) {
}


kernel void filter_s(
        int src_ns, int src_s0, int src_s1,
        int src_nt, int src_t0, int src_t1,

        int dst_ns, int dst_s0, int dst_s1,
        int dst_nt, int dst_t0, int dst_t1,

        float coord_scale_s,
        float tau0_s,
        float tau1_s,
        float scale_s,

        global const float* tmp,
        global float* output) {
}

