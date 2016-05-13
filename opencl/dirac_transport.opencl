// vim: filetype=opencl

float compute_rect(const float tau0, const float tau1, 
                   float l, float r) {
    l = clamp(l, tau0, tau1);
    r = clamp(r, tau0, tau1);
    return r - l;
}

/** This kernel filters in the `t` direction.
 *
 * Input data: [s_in t_in]
 * Kernel order: [32 8] in [s_active_in t_active_out]
 * Output data: [t_active_out s_active_in]
 *
 * Reads are coalesced and filter entries are computed without
 * thread divergence.
 */
kernel void filter_t(
        const int src_ns, const int src_s0, const int src_s1,
        const int src_nt, const int src_t0, const int src_t1,
        const float src_ds, const float src_ws,
        const float src_dt, const float src_wt,

        const int dst_ns, const int dst_s0, const int dst_s1,
        const int dst_nt, const int dst_t0, const int dst_t1,
        const float dst_ds, const float dst_ws,
        const float dst_dt, const float dst_wt,

        const float coord_scale_t,
        float tau0_t,
        float tau1_t,
        const float scale_t,

        global const float* input,
        global float* tmp) {
    const int is_src_off = get_global_id(0);
    const int it_dst_off = get_global_id(1);

    // bounds check
    if(is_src_off >= (src_s1 - src_s0) 
            || it_dst_off >= (dst_t1 - dst_t0)) {
        return;
    }

    const int is_src = is_src_off + src_s0;
    const int it_dst = it_dst_off + dst_t0;

    // compute taus for this value of t
    const float t_dst = (it_dst - dst_wt)*dst_dt;
    tau0_t += t_dst * coord_scale_t;
    tau1_t += t_dst * coord_scale_t;

    // find bounds 
    float t0_src = floor(tau0_t/src_dt + src_wt + 0.5f);
    float t1_src = ceil(tau1_t/src_dt + src_wt + 0.5f);

    t0_src = clamp(t0_src, (float)src_t0, (float)src_t1);
    t1_src = clamp(t1_src, (float)src_t0, (float)src_t1);

    float accum = 0.f;
    for(int it_src = t0_src; it_src < t1_src; ++it_src) {
        const float t_left = (it_src - src_wt - 0.5f)*src_dt;
        const float t_right = t_left + src_dt;
        const float w = compute_rect(tau0_t, tau1_t, t_left, t_right);
        accum += w * input[is_src + src_ns*it_src];
    }
    accum *= scale_t;

    tmp[it_dst_off + (src_s1 - src_s0)*is_src_off] = accum;
}

/** This kernel filters in the `s` direction.
 *
 * Input data: [s_active_in t_active_out]
 * Kernel order: [32 8] in [t_active_out s_active_out]
 * Output data: [s_active_out t_active_out] written into [s_out t_out]
 *
 * Reads are coalesced and filter entries are computed without 
 * thread divergence.
 */
kernel void filter_s(
        const int src_ns, const int src_s0, const int src_s1,
        const int src_nt, const int src_t0, const int src_t1,
        const float src_ds, const float src_ws,
        const float src_dt, const float src_wt,

        const int dst_ns, const int dst_s0, const int dst_s1,
        const int dst_nt, const int dst_t0, const int dst_t1,
        const float dst_ds, const float dst_ws,
        const float dst_dt, const float dst_wt,

        const float coord_scale_s,
        float tau0_s,
        float tau1_s,
        const float scale_s,

        global const float* tmp,
        global float* dst) {
    const int is_src_off = get_global_id(0);
    const int it_dst_off = get_global_id(1);

    // bounds check
    if(is_src_off >= (src_s1 - src_s0) 
            || it_dst_off >= (dst_t1 - dst_t0)) {
        return;
    }

    const int is_src = is_src_off + src_s0;
    const int it_dst = it_dst_off + dst_t0;

    // compute taus for this value of t
    const float t_dst = (it_dst - dst_wt)*dst_dt;
    tau0_t += t_dst * coord_scale_t;
    tau1_t += t_dst * coord_scale_t;

    // find bounds 
    float t0_src = floor(tau0_t/src_dt + src_wt + 0.5f);
    float t1_src = ceil(tau1_t/src_dt + src_wt + 0.5f);

    t0_src = clamp(t0_src, (float)src_t0, (float)src_t1);
    t1_src = clamp(t1_src, (float)src_t0, (float)src_t1);

    float accum = 0.f;
    for(int it_src = t0_src; it_src < t1_src; ++it_src) {
        const float t_left = (it_src - src_wt - 0.5f)*src_dt;
        const float t_right = t_left + src_dt;
        const float w = compute_rect(tau0_t, tau1_t, t_left, t_right);
        accum += w * input[is_src + src_ns*it_src];
    }
    accum *= scale_t;

    tmp[it_dst_off + (src_s1 - src_s0)*is_src_off] = accum;
}

