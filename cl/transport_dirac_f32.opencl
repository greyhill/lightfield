// vim: filetype=opencl

inline float Rect_integrate(const float tau0, const float tau1,
        float l, float r) {
    l = fmin(fmax(l, tau0), tau1);
    r = fmin(fmax(r, tau0), tau1);
    return r - l;
}

float transport_t_iprod(
        const int,
        const int, const int,
        ImageGeometry, ImageGeometry,
        const int, const int,
        const int, const int,

        const int, const int,
        const int, const int,

        const float, const float, const float, const float,
        global float*);

float transport_s_iprod(
        const int,
        const int, const int,
        ImageGeometry, ImageGeometry,
        const int, const int,
        const int, const int,

        const int, const int,
        const int, const int,

        const float, const float, const float, const float,
        global float*);

kernel void transport_t(
        ImageGeometry src_geom,
        ImageGeometry dst_geom,

        const int src_is0, const int src_is1, 
        const int src_it0, const int src_it1,

        const int dst_is0, const int dst_is1,
        const int dst_it0, const int dst_it1,

        const float d_scale, const float base_tau0, const float base_tau1, const float h,
        global float* src,
        global float* tmp) {
    const int src_is_offset = get_global_id(0);
    const int dst_it_offset = get_global_id(1);

    local float value_cache[32*8];
    local int coord_cache[32*8];
    const int local_id = get_local_id(0) + 32*get_local_id(1);
    const int local_id_t = get_local_id(1) + 8*get_local_id(0);

    if(src_is_offset >= src_is1 - src_is0 || dst_it_offset >= dst_it1 - dst_it0) {
        value_cache[local_id] = 0.f;
        coord_cache[local_id] = -1;
    } else {
        const int src_is = src_is_offset + src_is0;
        const int dst_it = dst_it_offset + dst_it0;

        value_cache[local_id] = transport_t_iprod(src_is,
                src_is_offset, dst_it_offset,

                src_geom,
                dst_geom,

                src_is0, src_is1,
                src_it0, src_it1,

                dst_is0, dst_is1,
                dst_it0, dst_it1,

                d_scale, base_tau0, base_tau1, h,
                src);
        coord_cache[local_id] = dst_it_offset + (dst_it1 - dst_it0)*src_is_offset;
    }

    // Do coalesced write after transpose in shared memory
    barrier(CLK_LOCAL_MEM_FENCE);
    const int write_coord = coord_cache[local_id_t];
    const float write_val = value_cache[local_id_t];
    if(write_coord >= 0) {
        tmp[write_coord] = write_val;
    }
}

kernel void transport_s(
        ImageGeometry src_geom,
        ImageGeometry dst_geom,

        int src_is0, int src_is1, 
        int src_it0, int src_it1,

        int dst_is0, int dst_is1,
        int dst_it0, int dst_it1,

        const float d_scale, const float base_tau0, const float base_tau1, const float h,
        global float* tmp,
        global float* dst,
        
        int conservative,
        int overwrite) {
    const int dst_it_offset = get_global_id(0);
    const int dst_is_offset = get_global_id(1);

    local float value_cache[32*8];
    local int coord_cache[32*8];
    const int local_id = get_local_id(0) + 32*get_local_id(1);
    const int local_id_t = get_local_id(1) + 8*get_local_id(0);

    if(dst_it_offset >= (dst_it1 - dst_it0) || dst_is_offset >= (dst_is1 - dst_is0)) {
        value_cache[local_id] = 0.f;
        coord_cache[local_id] = -1;
    } else {
        const int dst_it = dst_it_offset + dst_it0;
        const int dst_is = dst_is_offset + dst_is0;

        value_cache[local_id] = transport_s_iprod(dst_it,
                dst_it_offset, dst_is_offset,
                src_geom,
                dst_geom,

                src_is0, src_is1,
                src_it0, src_it1,

                dst_is0, dst_is1,
                dst_it0, dst_it1,

                d_scale, base_tau0, base_tau1, h,
                tmp);

        coord_cache[local_id] = dst_is + dst_geom->ns*dst_it;
    }

    // Do coalesced write after transpose in shared memory
    barrier(CLK_LOCAL_MEM_FENCE);
    const int write_coord = coord_cache[local_id_t];
    const float write_val = value_cache[local_id_t];
    if(write_coord >= 0) {
        if(!conservative || (write_val != 0.f)) {
            if(overwrite) {
                dst[write_coord] = write_val;
            } else {
                dst[write_coord] += write_val;
            }
        }
    }
}

float transport_t_iprod(
        const int src_is,
        const int src_is_offset, const int dst_it_offset,

        ImageGeometry src_geom,
        ImageGeometry dst_geom,

        const int src_is0, const int src_is1, 
        const int src_it0, const int src_it1,

        const int dst_is0, const int dst_is1,
        const int dst_it0, const int dst_it1,

        const float d_scale, const float base_tau0, const float base_tau1, const float h,
        global float* src) {
    // compute taus for this row
    const int dst_it = dst_it_offset + dst_it0;
    const float dst_t = ImageGeometry_it2t(dst_geom, dst_it);
    const float tau0 = base_tau0 + dst_t*d_scale;
    const float tau1 = base_tau1 + dst_t*d_scale;

    // compute integral coordinates
    int src_itmin = floor(ImageGeometry_t2it(src_geom, tau0));
    int src_itmax = ceil(ImageGeometry_t2it(src_geom, tau1));
    src_itmin = max(min(src_itmin, src_it1), src_it0);
    src_itmax = max(min(src_itmax, src_it1), src_it0);

    // TODO consolidate some operations
    float accum = 0.f;
    for(int src_it=src_itmin; src_it<src_itmax; ++src_it) {
        const float src_t = ImageGeometry_it2t(src_geom, src_it);
        const float w = Rect_integrate(tau0, tau1, 
                src_t - src_geom->dt/2.f,
                src_t + src_geom->dt/2.f);
        accum += w * src[src_is + src_geom->ns*src_it];
    }

    return accum * h;
}

float transport_s_iprod(
        const int dst_it,
        const int dst_it_offset, const int dst_is_offset,
        ImageGeometry src_geom,
        ImageGeometry dst_geom,

        int src_is0, int src_is1, 
        int src_it0, int src_it1,

        int dst_is0, int dst_is1,
        int dst_it0, int dst_it1,

        const float d_scale, const float base_tau0, const float base_tau1, const float h,
        global float* tmp) {
    // compute taus for this row
    const int dst_is = dst_is_offset + dst_is0;
    const float dst_s = ImageGeometry_is2s(dst_geom, dst_is);
    const float tau0 = base_tau0 + dst_s*d_scale;
    const float tau1 = base_tau1 + dst_s*d_scale;

    // compute integral coordinates
    int src_ismin = floor(ImageGeometry_s2is(src_geom, tau0));
    int src_ismax = ceil(ImageGeometry_s2is(src_geom, tau1));
    src_ismin = max(min(src_ismin, src_is1), src_is0);
    src_ismax = max(min(src_ismax, src_is1), src_is0);

    // TODO consolidate some operations
    float accum = 0.f;
    for(int src_is=src_ismin; src_is<src_ismax; ++src_is) {
        const float src_s = ImageGeometry_is2s(src_geom, src_is);
        const float w = Rect_integrate(tau0, tau1, 
                src_s - src_geom->ds/2.f,
                src_s + src_geom->ds/2.f);
        accum += w * tmp[dst_it_offset + (dst_it1 - dst_it0)*(src_is - src_is0)];
    }

    return accum * h;
}

