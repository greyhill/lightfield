// vim: filetype=opencl

struct ImageGeometry {
    int ns;
    int nt;
    float ds;
    float dt;
    float offset_s;
    float offset_t;
    float ws;
    float wt;
};
typedef constant struct ImageGeometry* ImageGeometry;

inline float ImageGeometry_is2s(ImageGeometry self, const int is) {
    return (is - self->ws)*self->ds;
}

inline float ImageGeometry_s2is(ImageGeometry self, float s) {
    return s/self->ds + self->ws + 0.5f;
}

inline float ImageGeometry_it2t(ImageGeometry self, const int it) {
    return (it - self->wt)*self->dt;
}

inline float ImageGeometry_t2it(ImageGeometry self, float t) {
    return t/self->dt + self->wt + 0.5f;
}

