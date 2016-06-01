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

