struct LFTransport {
    // borrowed pointers
    const struct LFPlaneGeometry* src_plane;
    const struct LFPlaneGeometry* dst_plane;
    const struct LFAngularPlane* angular_plane;
    const struct LFOptics* src2plane_x;
    const struct LFOptics* src2plane_y;
    const struct LFOptics* dst2plane_x;
    const struct LFOptics* dst2plane_y;

    // owned
    struct LFOptics src2dst_x;
    struct LFOptics src2dst_y;
    struct LFOptics dst2src_x;
    struct LFOptics dst2src_y;
};

extern void LFTransport_init(struct LFTransport* x);
extern void LFTransport_setup(struct LFTransport* x,
        const struct LFPlaneGeometry* src_plane,
        const struct LFPlaneGeometry* dst_plane,
        const struct LFAngularPlane* angular_plane,
        const struct LFOptics* src2plane_x,
        const struct LFOptics* src2plane_y,
        const struct LFOptics* dst2plane_x,
        const struct LFOptics* dst2plane_y);

