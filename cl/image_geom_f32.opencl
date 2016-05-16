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

