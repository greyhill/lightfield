#pragma once

// One-dimensional affine optical transformation
struct LFOptics {
    float pp;
    float pa;
    float ap;
    float aa;
    float cp;
    float ca;
};

extern void LFOptics_identity(struct LFOptics* optics);
extern void LFOptics_translation(struct LFOptics* optics, const float q);
extern void LFOptics_refraction(struct LFOptics* optics, const float f, const float c);
extern void LFOptics_compose(const struct LFOptics* lhs, const struct LFOptics* rhs, struct LFOptics* out);
extern void LFOptics_invert(const struct LFOptics* optics, struct LFOptics* out);
extern void LFOptics_ray(const struct LFOptics* optics, 
        const float p, const float a,
        float* p_out, float* a_out);

