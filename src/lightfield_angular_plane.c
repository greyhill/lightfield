#include "lightfield/lightfield.h"

void LFAngularPlane_init(struct LFAngularPlane* plane) {
    plane->du = NAN;
    plane->dv = NAN;
    plane->type = LF_PLANE_UNINIT;
    plane->mode = (enum LFAngularPlaneMode)LF_PLANE_UNINIT;
    plane->num_points = 0;
    plane->u_points = NULL;
    plane->v_points = NULL;
    plane->w_points = NULL;
}

bool LFAngularPlane_setup(struct LFAngularPlane* plane,
                          float du,
                          float dv,
                          enum LFAngularPlaneType type,
                          enum LFAngularPlaneMode mode,
                          const size_t num_points,
                          const float* u_points,
                          const float* v_points,
                          const float* w_points) {
    bool ok = true;

    plane->du = du;
    plane->dv = dv;
    plane->type = type;
    plane->mode = mode;
    plane->num_points = num_points;

    if(plane->u_points) free(plane->u_points);
    plane->u_points = malloc(sizeof(float)*plane->num_points);
    LF_TRY(plane->u_points != NULL);
    memcpy(plane->u_points, u_points, sizeof(float)*plane->num_points);

    if(plane->v_points) free(plane->v_points);
    plane->v_points = malloc(sizeof(float)*plane->num_points);
    LF_TRY(plane->v_points != NULL);
    memcpy(plane->v_points, v_points, sizeof(float)*plane->num_points);

    if(plane->w_points) free(plane->w_points);
    plane->w_points = malloc(sizeof(float)*plane->num_points);
    LF_TRY(plane->w_points != NULL);
    memcpy(plane->w_points, w_points, sizeof(float)*plane->num_points);

    if(0) {
err:
        ok = false;
    }

    return ok;
}

void LFAngularPlane_del(struct LFAngularPlane* plane) {
    plane->du = NAN;
    plane->dv = NAN;
    plane->type = LF_PLANE_UNINIT;
    plane->mode = (enum LFAngularPlaneMode)LF_PLANE_UNINIT;
    plane->num_points = 0;
    if(plane->u_points) free(plane->u_points);
    plane->u_points = NULL;
    if(plane->v_points) free(plane->v_points);
    plane->v_points = NULL;
    if(plane->w_points) free(plane->w_points);
    plane->w_points = NULL;
}

