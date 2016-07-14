#include "lightfield/lightfield.h"
#include <stdlib.h>
#include "CL/cl.h"

struct LFEnvironment {
    cl_context context;
    cl_command_queue queue;
    // ... TODO
};

struct LFTransport {
    // TODO
};

void LFOpticalX_identity(struct LFOpticalX* x) {
    (void)x;
    // TODO;
}

void LFOpticalX_compose(
        const struct LFOpticalX* lhs,
        const struct LFOpticalX* rhs,
        struct LFOpticalX* out) {
    (void)lhs;
    (void)rhs;
    (void)out;
    // TODO;
}

void LFOpticalX_translation(
        struct LFOpticalX* x,
        const float distance) {
    (void)x;
    (void)distance;
    // TODO
}

void LFOpticalX_lens(
        struct LFOpticalX* x,
        const float center_x,
        const float center_y,
        const float focal_length) {
    (void)x;
    (void)center_x;
    (void)center_y;
    (void)focal_length;
    // TODO
}

void LFOpticalX_invert(
        const struct LFOpticalX* x,
        struct LFOpticalX* out) {
    (void)x;
    (void)out;
    // TODO
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

bool LFTransport_back_view(
        struct LFTransport* xport,
        const float* dst,
        float* src,
        size_t angle) {
    (void)xport;
    (void)src;
    (void)dst;
    (void)angle;

    return true; // TODO
}

