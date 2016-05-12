struct LFTransport {
    // borrowed
    const struct LFAngularPlane* angular_plane;
    const struct LFPlaneGeometry* src_plane;
    const struct LFOptics* src_to_root_s;
    const struct LFOptics* src_to_root_t;

    const struct LFPlaneGeometry* dst_plane;
    const struct LFOptics* dst_to_root_s;
    const struct LFOptics* dst_to_root_t;

    // owned
    float scale;
    struct LFOptics src_to_dst_s;
    struct LFOptics src_to_dst_t;
};

extern bool LFTransport_init(struct LFTransport* x);
extern bool LFTransport_del(struct LFTransport* x);
extern bool LFTransport_setup(struct LFTransport* x,
        const struct LFAngularPlane* angular_plane,
        const struct LFPlaneGeometry* src_plane,
        const struct LFOptics* src_to_root_s,
        const struct LFOptics* src_to_root_t,
        const struct LFPlaneGeometry* dst_plane,
        const struct LFOptics* dst_to_root_s,
        const struct LFOptics* dst_to_root_t,
        const float scale);

extern size_t LFTransport_tmp_size(const struct LFTransport* x);

extern bool LFTransport_compute(struct LFTransport* x,
        size_t i_view, 
        cl_mem src, cl_mem dst, cl_mem tmp);

