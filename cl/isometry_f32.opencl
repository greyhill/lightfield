// vim: filetype=opencl

struct Isometry {
    float3 s;
    float3 t;
    float3 d;
    float3 position;
};
typedef constant struct Isometry* Isometry;

