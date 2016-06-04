// vim: filetype=opencl
//
// n.b., this file is customarily built with transport_pillbox_f32.opencl,
// image_geom_f32.opencl, light_volume_f32.opencl, optics_f32.opencl,
// spline_kernel_f32.opencl

kernel void volume_forw_t(
        LightVolume volume_geom,
        ImageGeometry slice_geom,
        ImageGeometry dst_geom,
        
        global struct Optics* dst_to_slice,
        global struct TrapezoidSplineKernel* splines_t, 

        const int ia, const int na,
        const float u, const float v,
        const int iz,
        
        global const float* volume,
        global float* tmp) {
    const int src_is = get_global_id(0);
    const int dst_it = get_global_id(1);

    local float value_cache[32*8];
    local int coord_cache[32*8];
    const int local_id = get_local_id(0) + 32*get_local_id(1);
    const int local_id_t = get_local_id(1) + 8*get_local_id(0);

    global const float* slice = volume + slice_geom->ns*slice_geom->nt*iz;

    if(src_is >= slice_geom->ns || dst_it >= dst_geom->nt) {
        value_cache[local_id] = 0.f;
        coord_cache[local_id] = -1;
    } else {
        global struct TrapezoidSplineKernel* my_spline = splines_t + na*iz + ia;
        const float mag = my_spline->magnification;
        const float base_tau0 = my_spline->tau0;
        const float base_tau1 = my_spline->tau1;
        const float base_tau2 = my_spline->tau2;
        const float base_tau3 = my_spline->tau3;
        const float h = my_spline->height;

        value_cache[local_id] = transport_t_iprod(src_is,
                src_is, dst_it,

                slice_geom,
                dst_geom,

                0, slice_geom->ns,
                0, slice_geom->nt,

                0, dst_geom->ns,
                0, dst_geom->nt,

                mag, base_tau0, base_tau1, base_tau2, base_tau3, h,
                slice);
        coord_cache[local_id] = dst_it + dst_geom->nt*src_is;
    }

    // coalesced write after transpose in shared memory
    barrier(CLK_LOCAL_MEM_FENCE);
    const int write_coord = coord_cache[local_id_t];
    const float write_val = value_cache[local_id_t];
    if(write_coord >= 0) {
        tmp[write_coord] = write_val;
    }
}

kernel void volume_forw_s(
        LightVolume volume_geom,
        ImageGeometry slice_geom,
        ImageGeometry dst_geom,
        
        global struct Optics* dst_to_slice,
        global struct TrapezoidSplineKernel* splines_s,

        const int ia, const int na,
        const float u, const float v,
        const int iz,
        
        global const float* tmp,
        global float* dst) {
    const int dst_it = get_global_id(0);
    const int dst_is = get_global_id(1);

    local float value_cache[32*8];
    local float coord_cache[32*8];
    const int local_id = get_local_id(0) + 32*get_local_id(1);
    const int local_id_t = get_local_id(1) + 8*get_local_id(0);

    if(dst_it >= dst_geom->nt || dst_is >= dst_geom->ns) {
        value_cache[local_id] = 0.f;
        coord_cache[local_id] = -1;
    } else {
        global struct TrapezoidSplineKernel* my_spline = splines_s + na*iz + ia;
        const float mag = my_spline->magnification;
        const float base_tau0 = my_spline->tau0;
        const float base_tau1 = my_spline->tau1;
        const float base_tau2 = my_spline->tau2;
        const float base_tau3 = my_spline->tau3;
        const float h = my_spline->height;

        value_cache[local_id] = transport_s_iprod(dst_it,
                dst_it, dst_is,
                slice_geom,
                dst_geom,

                0, slice_geom->ns,
                0, slice_geom->nt,

                0, dst_geom->ns,
                0, dst_geom->nt,

                mag, base_tau0, base_tau1, base_tau2, base_tau3, h,
                tmp);
        coord_cache[local_id] = dst_is + dst_geom->ns * dst_it;
    }

    // coalesced write after transpose in shared memory
    barrier(CLK_LOCAL_MEM_FENCE);
    const int write_coord = coord_cache[local_id_t];
    const float write_val = value_cache[local_id_t];
    if(write_coord >= 0) {
        dst[write_coord] += write_val;
    }
}

kernel void volume_back_t(
        LightVolume volume_geom,
        ImageGeometry slice_geom,
        ImageGeometry dst_geom,
        
        global struct Optics* slice_to_dst,
        global struct TrapezoidSplineKernel* splines_t,
        
        const int ia, const int na,
        const float u, const float v,
        const int iz,
        
        global const float* dst,
        global float* tmp) {
    const int dst_is = get_global_id(0);
    const int src_it = get_global_id(1);

    local float value_cache[32*8];
    local int coord_cache[32*8];
    const int local_id = get_local_id(0) + 32*get_local_id(1);
    const int local_id_t = get_local_id(1) + 8*get_local_id(0);

    if(dst_is >= dst_geom->ns || src_it >= slice_geom->nt) {
        value_cache[local_id] = 0.f;
        coord_cache[local_id] = -1;
    } else {
        global struct TrapezoidSplineKernel* my_spline = splines_t + na*iz + ia;
        const float mag = my_spline->magnification;
        const float base_tau0 = my_spline->tau0;
        const float base_tau1 = my_spline->tau1;
        const float base_tau2 = my_spline->tau2;
        const float base_tau3 = my_spline->tau3;
        const float h = my_spline->height;

        value_cache[local_id] = transport_t_iprod(dst_is,
                dst_is, src_it,

                dst_geom,
                slice_geom,

                0, dst_geom->ns,
                0, dst_geom->nt,

                0, slice_geom->ns,
                0, slice_geom->nt,

                mag, base_tau0, base_tau1, base_tau2, base_tau3, h,
                dst);
        coord_cache[local_id] = src_it + slice_geom->nt*dst_is;
    }

    // coalesced write after transpose in shared memory
    barrier(CLK_LOCAL_MEM_FENCE);
    const int write_coord = coord_cache[local_id_t];
    const float write_val = value_cache[local_id_t];
    if(write_coord >= 0) {
        tmp[write_coord] = write_val;
    }
}

kernel void volume_back_s(
        LightVolume volume_geom,
        ImageGeometry slice_geom,
        ImageGeometry dst_geom,
        
        global struct Optics* slice_to_dst,
        global struct TrapezoidSplineKernel* splines_s,
        
        const int ia, const int na,
        const float u, const float v,
        const int iz,
        
        global const float* tmp,
        global float* vol) {
    const int src_it = get_global_id(0);
    const int src_is = get_global_id(1);

    local float value_cache[32*8];
    local float coord_cache[32*8];
    const int local_id = get_local_id(0) + 32*get_local_id(1);
    const int local_id_t = get_local_id(1) + 8*get_local_id(0);

    if(src_it >= slice_geom->nt || src_is >= slice_geom->ns) {
        value_cache[local_id] = 0.f;
        coord_cache[local_id] = -1;
    } else {
        float accum = 0.f;

        global struct TrapezoidSplineKernel* my_spline = splines_s + na*iz + ia;
        const float mag = my_spline->magnification;
        const float base_tau0 = my_spline->tau0;
        const float base_tau1 = my_spline->tau1;
        const float base_tau2 = my_spline->tau2;
        const float base_tau3 = my_spline->tau3;
        const float h = my_spline->height;

        accum += transport_s_iprod(src_it,
                src_it, src_is,
                dst_geom,
                slice_geom,

                0, dst_geom->ns,
                0, dst_geom->nt,

                0, slice_geom->ns,
                0, slice_geom->nt,

                mag, base_tau0, base_tau1, base_tau2, base_tau3, h,
                tmp);

        coord_cache[local_id] = src_is + slice_geom->ns * (src_it + slice_geom->nt * iz);
        value_cache[local_id] = accum;
    }

    // coalesced write after transpose in shared memory
    barrier(CLK_LOCAL_MEM_FENCE);
    const int write_coord = coord_cache[local_id_t];
    const float write_val = value_cache[local_id_t];
    if(write_coord >= 0) {
        vol[write_coord] += write_val;
    }
}

