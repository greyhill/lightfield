#include "lightfield/lightfield.h"

float LFPlaneGeometry_ws(const struct LFPlaneGeometry* pg) {
    return (pg->ns - 1.f)/2.f + pg->offset_s;
}

float LFPlaneGeometry_wt(const struct LFPlaneGeometry* pg) {
    return (pg->nt - 1.f)/2.f + pg->offset_t;
}

