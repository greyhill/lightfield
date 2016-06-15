// vim: filetype=opencl

struct RectSplineKernel {
    float height;
    float magnification;
    float tau0;
    float tau1;
};

struct TrapezoidSplineKernel {
    float height;
    float magnification;
    float tau0;
    float tau1;
    float tau2;
    float tau3;
};

struct QuadSplineKernel {
    float height;
    float magnification;
    float tau0;
    float tau1;
    float tau2;
    float tau3;
    float tau4;
    float tau5;
    float tau6;
    float tau7;
};

float TrapezoidSplineKernel_sample(
        global struct TrapezoidSplineKernel* k,
        const float loc,
        const float x) {
    const float tau0 = k->tau0 + loc * k->magnification;
    const float tau1 = k->tau1 + loc * k->magnification;
    const float tau2 = k->tau2 + loc * k->magnification;
    const float tau3 = k->tau3 + loc * k->magnification;

    if(x < tau0) {
        return 0.f;
    } else if(x < tau1) {
        return k->height*(x - tau0)/(tau1 - tau0);
    } else if(x < tau2) {
        return k->height;
    } else if(x < tau3) {
        return k->height*(tau3 - x)/(tau3 - tau2);
    } else {
        return 0.f;
    }
}

float TrapezoidSplineKernel_integrate(
        global struct TrapezoidSplineKernel* k,
        const float loc,
        const float li, 
        const float ri) {
    const float tau0 = k->tau0 + loc * k->magnification;
    const float tau1 = k->tau1 + loc * k->magnification;
    const float tau2 = k->tau2 + loc * k->magnification;
    const float tau3 = k->tau3 + loc * k->magnification;

    float accum = 0.f;

    float l = fmin(fmax(li, tau0), tau1);
    float r = fmin(fmax(ri, tau0), tau1);
    accum += ((r - tau0)*(r - tau0) - (l - tau0)*(l - tau0))/(2.f*(tau1 - tau0));

    l = fmin(fmax(li, tau1), tau2);
    r = fmin(fmax(ri, tau1), tau2);
    accum += r - l;

    l = fmin(fmax(li, tau2), tau3);
    r = fmin(fmax(ri, tau2), tau3);
    accum += ((l - tau3)*(l - tau3) - (r - tau3)*(r - tau3))/(2.f*(tau3 - tau2));

    return k->height * accum;
}

#define POW2(m) ((m)*(m))
#define POW3(m) ((m)*(m)*(m))
float QuadSplineKernel_integrate(
        global struct QuadSplineKernel* k,
        const float loc,
        const float x0,
        const float x1) {
    const float t0 = k->tau0 + loc * k->magnification;
    const float t1 = k->tau1 + loc * k->magnification;
    const float t2 = k->tau2 + loc * k->magnification;
    const float t3 = k->tau3 + loc * k->magnification;
    const float t4 = k->tau4 + loc * k->magnification;
    const float t5 = k->tau5 + loc * k->magnification;
    const float t6 = k->tau6 + loc * k->magnification;
    const float t7 = k->tau7 + loc * k->magnification;

    float accum = 0.f;
    float c1 = 1.f / ((t1 - t0) * ((t1 - t0) / 2.f + t2 - t1 + (t3 - t2) / 2.f));

    float l, r;

    l = fmax(x0, t0);
    r = fmin(x1, t1);
    if(r > l) {
        accum += c1 * (POW3(r - t0) - POW3(l - t0)) / 6.f;
    }

    l = fmax(x0, t1);
    r = fmin(x1, t2);
    if(r > l) {
        accum += c1 * (t1 - t0)*(POW2(r - t1) - POW2(l - t1)) / 2.f;
        accum += c1 * POW2(t1 - t0) * (r - l) / 2.f;
    }

    l = fmax(x0, t2);
    r = fmin(x1, t3);
    if(r > l) {
        accum += (r - l);
        accum -= c1 * (t1 - t0)*(POW3(r - t3) - POW3(l - t3)) / (6.f * (t3 - t2));
    }

    l = fmax(x0, t3);
    r = fmin(x1, t4);
    accum += fmax(0.f, r - l);

    l = fmax(x0, t4);
    r = fmin(x1, t5);
    if(r > l) {
        accum += (r - l);
        accum -= c1 * (t1 - t0)*(POW3(r - t4) - POW3(l - t4)) / (6.f * (t3 - t2));
    }

    l = fmax(x0, t5);
    r = fmin(x1, t6);
    if(r > l) {
        accum += c1 * POW2(t1 - t0) / 2.f * (r - l);
        accum -= c1 * (t1 - t0)/2.f * (POW2(r - t6) - POW2(l - t6));
    }

    l = fmax(x0, t6);
    r = fmin(x1, t7);
    if(r > l) {
        accum += c1 * (POW3(r - t7) - POW3(l - t7)) / 6.f;
    }

    return accum * k->height;
}
#undef POW2
#undef POW3


