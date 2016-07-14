/// Light transport API

#pragma once

#include <stdlib.h>
#include <stdbool.h>

enum LFAngularBasis {
    LFPillbox = 0,
    LFDirac = 1,
};

/// Optical transformation
struct LFOpticalX {
    float ss;
    float us;
    float su;
    float uu;

    float tt;
    float vt;
    float tv;
    float vv;

    float s;
    float t;
    float u;
    float v;
};

extern void
LFOpticalX_identity(struct LFOpticalX* x);

extern void
LFOpticalX_compose(const struct LFOpticalX* lhs, 
                   const struct LFOpticalX* rhs,
                   struct LFOpticalX* out);

extern void
LFOpticalX_translation(struct LFOpticalX* x,
                       const float distance);

extern void 
LFOpticalX_lens(struct LFOpticalX* x,
                const float center_x,
                const float center_y,
                const float focal_length);

extern void
LFOpticalX_invert(const struct LFOpticalX* x,
                  struct LFOpticalX* out);

/// Angular plane
struct LFAngularPlane {
    float ds;
    float dt;
    enum LFAngularBasis basis;

    // borrowed
    size_t num_points;
    const float* points_s;
    const float* points_t;
    const float* points_w;
};

/// Image (plane) geometry
struct LFImageGeometry {
    size_t ns;
    size_t nt;

    /// Pixel size
    float ds;
    /// Pixel size
    float dt;

    /// Unitless (pixel fractions)
    float offset_s;
    /// Unitless (pixel fractions)
    float offset_t;
};

/// Light field geometry
struct LFGeometry {
    struct LFImageGeometry geom;
    struct LFAngularPlane plane;
    struct LFOpticalX to_plane;
};

/// Opaque environment type
struct LFEnvironment;

/// Opaque light field transport type
struct LFTransport;

/// Create a new OpenCL environment, queue, etc.
extern struct LFEnvironment*
LFEnvironment_new();

extern void 
LFEnvironment_del(struct LFEnvironment* env);

extern struct LFTransport* 
LFTransport_new(
        const struct LFGeometry* source, 
        const struct LFGeometry* dest, 
        struct LFEnvironment* env);

extern void
LFTransport_del(struct LFTransport* xport);

extern bool
LFTransport_forw_view(
        struct LFTransport* xport,
        const float* src,
        float* dst,
        size_t angle);

extern bool
LFTransport_back_view(
        struct LFTransport* xport,
        const float* dst,
        float* src,
        size_t angle);

