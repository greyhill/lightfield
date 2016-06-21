// vim: filetype=opencl

#define PF_QUAD 0
#define PF_ABS 1
#define PF_FAIR 2

struct PotentialFunction {
    int type;
    union {
        struct {
            float weight;
        } quadratic;

        struct {
            float weight;
        } absolute_value;

        struct {
            float weight;
            float delta;
        } fair;
    } params;
};
typedef global struct PotentialFunction* PotentialFunction;

// Returns mu/2 (x - y)^2 + Pf(x)
float PotentialFunction_shrink(PotentialFunction pf,
                               float mu,
                               float y) {
    switch(pf->type) {
        case PF_QUAD:
            return mu*y / (mu + pf->params.quadratic.weight);

        case PF_ABS: {
            float wi = pf->params.absolute_value.weight / mu;
            return sign(y) * fmax(0.f, fabs(y) - wi);
         };

        case PF_FAIR:
            break;
    }
}

float PotentialFunction_grad(PotentialFunction pf,
                             float x) {
}

float PotentialFunction_huber(PotentialFunction pf,
                              float x) {
    switch(pf->type) {
        case PF_QUAD: 
            return pf->params.quadratic.weight;

        case PF_ABS: 
            return 1.f / fabs(x);

        case PF_FAIR: {
            const float axd = fabs(x / pf->params.fair.delta);
            return pf->params.fair.weight / (1.f + axd);
        };
    }
}

