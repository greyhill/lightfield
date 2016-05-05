struct LFPlaneGeometry {
    size_t ns;
    size_t nt;

    float ds;
    float dt;

    float offset_s;
    float offset_t;
};

extern float LFPlaneGeometry_ws(const struct LFPlaneGeometry* pg);
extern float LFPlaneGeometry_wt(const struct LFPlaneGeometry* pg);
