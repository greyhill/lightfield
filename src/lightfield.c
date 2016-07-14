#include "lightfield/lightfield.h"
#include <stdlib.h>

struct LFEnvironment {
    // TODO
};

struct LFTransport {
    // TODO
};

void LFOpticalX_identity(struct LFOpticalX* x) {
    x->ss = 1;
    x->su = 0;
    x->us = 0;
    x->uu = 1;

    x->tt = 1;
    x->tv = 0;
    x->vt = 0;
    x->vv = 1;
}

void LFOpticalX_compose(
        const struct LFOpticalX* lhs,
        const struct LFOpticalX* rhs,
        struct LFOpticalX* out) {
    out->ss = lhs->ss * rhs->ss + lhs->su * rhs->us;
    out->su = lhs->ss * rhs->su + lhs->su * rhs->uu;
    out->us = lhs->us * rhs->ss + lhs->uu * rhs->us;
    out->uu = lhs->us * rhs->su + lhs->uu * rhs->uu;

    out->tt = lhs->tt * rhs->tt + lhs->tv * rhs->vt;
    out->tv = lhs->tt * rhs->tv + lhs->tv * rhs->vv;
    out->vt = lhs->vt * rhs->tt + lhs->vv * rhs->vt;
    out->vv = lhs->vt * rhs->tv + lhs->vv * rhs->vv;

    out->s = lhs->s + lhs->ss * rhs->s + lhs->su * rhs->u;
    out->u = lhs->u + lhs->us * rhs->s + lhs->uu * rhs->u;
    out->t = lhs->t + lhs->tt * rhs->t + lhs->tv * rhs->v;
    out->v = lhs->v + lhs->vt * rhs->t + lhs->vv * rhs->v;
}

void LFOpticalX_translation(
        struct LFOpticalX* x,
        const float distance) {
    x->ss = 1;
    x->su = distance;
    x->us = 0;
    x->uu = 1;

    x->tt = 1;
    x->tv = distance;
    x->vt = 0;
    x->vv = 1;

    x->s = 0;
    x->t = 0;
    x->u = 0;
    x->v = 0;
}

void LFOpticalX_lens(
        struct LFOpticalX* x,
        const float center_x,
        const float center_y,
        const float focal_length) {
    x->ss = 1;
    x->su = 0;
    x->us = -1 / focal_length;
    x->uu = 1;

    x->tt = 1;
    x->tv = 0;
    x->vt = -1 / focal_length;
    x->vv = 1;

    x->s = 0;
    x->t = 0;
    x->u = center_x / focal_length;
    x->v = center_y / focal_length;
}

void LFOpticalX_invert(
        const struct LFOpticalX* x,
        struct LFOpticalX* out) {
    float s_det = x->ss * x->uu - x->su * x->us;
    out->ss = x->uu / s_det;
    out->su = -x->su / s_det;
    out->us = -x->us / s_det;
    out->uu = x->ss / s_det;
    out->s = -(out->ss * x->s + out->su * x->u);
    out->u = -(out->us * x->s + out->uu * x->u);

    float t_det = x->tt * x->vv - x->tv * x->vt;
    out->tt = x->vv / t_det;
    out->tv = -x->tv / t_det;
    out->vt = -x->vt / t_det;
    out->vv = x->tt / t_det;
    out->t = -(out->tt * x->t + out->tv * x->v);
    out->v = -(out->vt * x->t + out->vv * x->v);
}

struct LFEnvironment* LFEnvironment_new() {
    struct LFEnvironment* to_return = NULL;
    to_return = malloc(sizeof(*to_return));
    return to_return;
}

void LFEnvironment_del(struct LFEnvironment* env) {
    free(env);
}

struct LFTransport* LFTransport_new(
        const struct LFGeometry* source,
        const struct LFGeometry* dest,
        struct LFEnvironment* env) {
    (void)source;
    (void)dest;
    (void)env;

    struct LFTransport* to_return = NULL;
    to_return = malloc(sizeof(*to_return));
    return to_return; // raise failed allocation as NULL ptr
}

bool LFTransport_forw_view(
        struct LFTransport* xport,
        const float* src,
        float* dst,
        size_t angle) {
    (void)xport;
    (void)src;
    (void)dst;
    (void)angle;

    return true; // TODO
}

