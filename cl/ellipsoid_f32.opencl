// vim: filetype=opencl

struct Ellipsoid {
    float xx;
    float xy;
    float xz;
    float xr;
    float xc;

    float yx;
    float yy;
    float yz;
    float yr;
    float yc;

    float zx;
    float zy;
    float zz;
    float zr;
    float zc;

    float value;
};
typedef constant struct Ellipsoid* Ellipsoid;

/* Returns the value of this ellipsoid if the given point hits it;
 * otherwise, returns 0 */
float Ellipsoid_eval(Ellipsoid e, float x, float y, float z) {
    const float px = (e->xx*x + e->xy*y + e->zz*z - e->xc) / e->xr;
    const float py = (e->yx*x + e->yy*y + e->yz*z - e->yc) / e->yr;
    const float pz = (e->zx*x + e->zy*y + e->zz*z - e->zc) / e->zr;
    if(px*py + py*py + pz*pz < 1.f) {
        return e->value;
    } else {
        return 0.f;
    }
}

