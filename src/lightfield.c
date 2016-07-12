#include "lightfield/lightfield.h"
#include <stdlib.h>

struct LFEnvironment {
    // TODO
};

struct LFTransport {
    // TODO
};

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

