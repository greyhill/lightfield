// vim: filetype=opencl

struct LightVolume {
    int nx;
    int ny;
    int nz;

    float dx;
    float dy;
    float dz;

    float offset_x;
    float offset_y;
    float offset_z;

    float wx;
    float wy;
    float wz;
};
typedef constant struct LightVolume* LightVolume;

