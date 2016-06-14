// vim: filetype=opencl

struct Isometry {
    float3 x;
    float3 y;
    float3 z;
    float3 position;
};
typedef constant struct Isometry* Isometry;

