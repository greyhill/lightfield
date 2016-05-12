#include "lightfield/lightfield.h"

static cl_context lfcl_context = NULL;
static cl_command_queue lfcl_command_queue = NULL;

bool LFCL_simple_init() {
    LF_ERROR_START;
    cl_uint num_platforms = 0;
    cl_platform_id* platforms = NULL;
    cl_uint num_devices = 0;
    cl_device_id* devices = NULL;

    // enumerate platform IDs
    cl_err = clGetPlatformIDs(0, NULL, &num_platforms); 
    LF_CHECK_CL;
    platforms = malloc(sizeof(platforms) * num_platforms);
    LF_TRY(platforms != NULL);
    cl_err = clGetPlatformIDs(num_platforms, platforms, NULL); 
    LF_CHECK_CL;

    // enumerate devices
    cl_err = clGetDeviceIDs(platforms[0], CL_DEVICE_TYPE_GPU, 
            0, NULL, &num_devices);
    LF_CHECK_CL;
    devices = malloc(sizeof(devices) * num_devices);
    LF_TRY(devices != NULL);
    cl_err = clGetDeviceIDs(platforms[0], CL_DEVICE_TYPE_GPU,
            num_devices, devices, NULL);

    // create a context 
    lfcl_context = clCreateContext(NULL, 1, devices, NULL, NULL, &cl_err);
    LF_CHECK_CL;

    // create a command queue on that context
    lfcl_command_queue = clCreateCommandQueue(lfcl_context,
            devices[0], 0, &cl_err);
    LF_CHECK_CL;

    LF_ERROR_BLOCK;

    if(platforms) free(platforms);
    if(devices) free(devices);

    return ok;
}

cl_context LFCL_get_context() {
    if(lfcl_context == NULL) {
        LFCL_simple_init();
    }
    assert(lfcl_context != NULL);
    return lfcl_context;
}

cl_command_queue LFCL_get_queue() {
    if(lfcl_command_queue == NULL) {
        LFCL_simple_init();
    }
    assert(lfcl_context != NULL);
    return lfcl_command_queue;
}

