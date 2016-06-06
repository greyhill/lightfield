// vim: filetype=opencl

struct LightVolume {
    int nx;
    int ny;
    int nz;

    float dx;
    float dy;
    float dz;

    float offset_x;
    float offset_y;
    float offset_z;

    float wx;
    float wy;
    float wz;
};
typedef constant struct LightVolume* LightVolume;

kernel void volume_zero(
        LightVolume geom,
        global float* vol) {
    int ix = get_global_id(0);
    int iy = get_global_id(1);
    int iz = get_global_id(2);

    if(ix >= geom->nx || iy >= geom->ny || iz >= geom->nz) {
        return;
    }

    vol[ix + geom->nx*(iy + geom->ny*iz)] = 0.f;
}

kernel void volume_scale(
        ImageGeometry dst_geom,
        Optics optics_to_plane,
        Optics optics_to_object,
        const float s_plane, const float t_plane,
        global const float* input,
        global float* output,
        int overwrite) {
    const int is = get_global_id(0);
    const int it = get_global_id(1);

    if(is >= dst_geom->ns || it >= dst_geom->nt) {
        return;
    }

    // compute the ray leaving the current plane to hit at (s_plane, t_plane)
    const float s = ImageGeometry_is2s(dst_geom, is);
    const float t = ImageGeometry_it2t(dst_geom, it);
    const float4 ray = Optics_hit(optics_to_plane, s, t, s_plane, t_plane);

    // compute the ray after 
    const float4 ray_out = Optics_apply(optics_to_object, ray);
    const float3 ray3 = { 1.f, ray_out.s1, ray_out.s3 };
    if(overwrite) {
        output[is + dst_geom->ns*it] = length(ray3) * input[is + dst_geom->ns*it];
    } else {
        output[is + dst_geom->ns*it] += length(ray3) * input[is + dst_geom->ns*it];
    }
}

