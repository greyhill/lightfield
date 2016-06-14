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

