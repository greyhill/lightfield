enum LFAngularPlaneType {
    LF_PLANE_UNINIT = 0,
    LF_PLANE_DIRAC = 1,
    LF_PLANE_BOX = 2,
};

enum LFAngularPlaneCoordinate {
    LF_PLANE_SPATIAL = 4,
    LF_PLANE_ANGULAR = 8,
};

struct LFAngularPlane {
    float du;
    float dv;
    enum LFAngularPlaneType type;
    enum LFAngularPlaneCoordinate coordinate;

    // Owned points
    size_t num_points;
    float* u_points;
    float* v_points;
    float* w_points;
};

extern void LFAngularPlane_init(struct LFAngularPlane* plane);
extern bool LFAngularPlane_setup(struct LFAngularPlane* plane,
                                 const float du,
                                 const float dv,
                                 enum LFAngularPlaneType type,
                                 enum LFAngularPlaneCoordinate coordinate,
                                 const size_t num_points,
                                 const float* u_points,
                                 const float* v_points,
                                 const float* w_points);
extern void LFAngularPlane_del(struct LFAngularPlane* plane);

