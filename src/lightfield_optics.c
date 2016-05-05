#include "lightfield/lightfield.h"

void LFOptics_identity(struct LFOptics* optics) {
    optics->pp = 1.f;
    optics->pa = 0.f;
    optics->ap = 0.f;
    optics->aa = 1.f;
    optics->cp = 0.f;
    optics->ca = 0.f;
}

void LFOptics_translation(struct LFOptics* optics, const float q) {
    optics->pp = 1.f;
    optics->pa = q;
    optics->ap = 0.f;
    optics->aa = 1.f;
    optics->cp = 0.f;
    optics->ca = 0.f;
}

void LFOptics_refraction(struct LFOptics* optics, const float f, const float c) {
    optics->pp = 1.f;
    optics->pa = 0.f;
    optics->ap = - 1.f / f;
    optics->aa = 1.f;
    optics->cp = 0.f;
    optics->ca = c / f;
}

void LFOptics_compose(const struct LFOptics* lhs, const struct LFOptics* rhs, struct LFOptics* out) {
    float pp, pa, ap, aa, cp, ca;
    LFOptics_ray(lhs, rhs->pp, rhs->ap, &pp, &ap);
    LFOptics_ray(lhs, rhs->pa, rhs->aa, &pa, &aa);
    LFOptics_ray(lhs, rhs->cp, rhs->ca, &cp, &ca);
    out->pp = pp;
    out->pa = pa;
    out->ap = ap;
    out->aa = aa;
    out->cp = cp;
    out->ca = ca;
}

void LFOptics_invert(const struct LFOptics* optics, struct LFOptics* out) {
    const float d = optics->pp * optics->aa - optics->pa * optics->ap;
    const float pp = optics->aa / d;
    const float aa = optics->pp / d;
    const float pa = - optics->pa / d;
    const float ap = - optics->ap / d;
    const float cp = - (pp * optics->cp + pa * optics->ca);
    const float ca = - (ap * optics->cp + aa * optics->ca);
    out->pp = pp;
    out->pa = pa;
    out->ap = ap;
    out->aa = aa;
    out->cp = cp;
    out->ca = ca;
}

void LFOptics_ray(const struct LFOptics* optics,
        const float p, const float a,
        float* p_out, float* a_out) {
    *p_out = optics->pp * p + optics->pa * a + optics->cp;
    *a_out = optics->ap * p + optics->aa * a + optics->ca;
}

