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
typedef constant struct Optics* optics;

