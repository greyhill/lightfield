// vim: filetype=opencl

struct Optics {
    float ss;
    float su;
    float us;
    float uu;

    float tt;
    float tv;
    float vt;
    float vv;

    float s;
    float t;
    float u;
    float v;
};
typedef constant struct Optics* Optics;

/* Applies this optical transform to a point.  The float4 coords
 * is in the following format: { s u t v } */
float4 Optics_apply(Optics optics, float4 coords) {
    float4 to_return = {
        optics->ss * coords.s0 + optics->su * coords.s1 + optics->s,
        optics->us * coords.s0 + optics->uu * coords.s1 + optics->u,
        optics->tt * coords.s2 + optics->tv * coords.s3 + optics->t,
        optics->vt * coords.s2 + optics->vv * coords.s3 + optics->v,
    };
    return to_return;
}

/* Given an optical transformation and a (s,t) value, find the (u,v)
 * values to hit the optical plane at the given points (s_plane, t_plane).
 * The returned float4 coords are in { s u t v } order; see Optics_apply */
float4 Optics_hit(Optics optics, float s, float t, 
                  float s_plane, float t_plane) {
    float4 to_return = {
        s,
        (s_plane - optics->ss * s - optics->s)/optics->su,
        t,
        (t_plane - optics->tt * t - optics->t)/optics->tv,
    };
    return to_return;
}

