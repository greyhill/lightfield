#include "lightfield/lightfield.h"

static cl_context lfcl_context = NULL;
static cl_command_queue lfcl_command_queue = NULL;
static cl_device_id lfcl_device = NULL;

static cl_program dirac_transport_program = NULL;
static cl_kernel dirac_transport_filter_t = NULL;
static cl_kernel dirac_transport_filter_s = NULL;

static cl_program pillbox_transport_program = NULL;

static bool LFCL_build_programs();
static bool LFCL_build_dirac_transport();
static bool LFCL_build_pillbox_transport();

bool LFCL_simple_init() {
    if(lfcl_context != NULL && lfcl_command_queue != NULL) return true;

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
    LF_CHECK_CL;
    lfcl_device = devices[0];

    // create a context 
    lfcl_context = clCreateContext(NULL, 1, &lfcl_device, NULL, NULL, &cl_err);
    LF_CHECK_CL;

    // create a command queue on that context
    lfcl_command_queue = clCreateCommandQueue(lfcl_context,
            devices[0], 0, &cl_err);
    LF_CHECK_CL;

    LF_TRY(LFCL_build_programs());

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

bool LFCL_build_programs() {
    LF_ERROR_START;

    LF_TRY(LFCL_build_dirac_transport());
    LF_TRY(LFCL_build_pillbox_transport());

    LF_ERROR_BLOCK;

    return ok;
}

static bool LFCL_compile(cl_program program) {
    cl_int cl_err = CL_SUCCESS;
    cl_err = clBuildProgram(program, 0, NULL, "", NULL, NULL);
    if(cl_err == CL_SUCCESS) {
        return true;
    } else {
        // get the compiler log on a failure and print it out
        fprintf(stderr, "Failed to build OpenCL program\n");
        char* log = NULL;
        size_t log_size = 0;
        cl_err = clGetProgramBuildInfo(program, lfcl_device,
                CL_PROGRAM_BUILD_LOG,
                0, NULL, &log_size);
        if(cl_err != CL_SUCCESS) {
            fprintf(stderr, "Error getting build log size :-(\n");
            return false;
        }
        log = malloc(log_size);
        if(log == NULL) {
            fprintf(stderr, "Error in malloc() for log... is the world on fire?\n");
            return false;
        }
        cl_err = clGetProgramBuildInfo(program, lfcl_device,
                CL_PROGRAM_BUILD_LOG,
                log_size, log, NULL);
        fprintf(stderr, "Error log from compiler:\n%s\n", log);
        free(log);
        return false;
    }
}

extern int opencl_dirac_transport_opencl_len;
extern unsigned char opencl_dirac_transport_opencl[];
bool LFCL_build_dirac_transport() {
    LF_ERROR_START;

    // build program
    size_t source_len = opencl_dirac_transport_opencl_len;
    const char* source = (const char*)opencl_dirac_transport_opencl;
    dirac_transport_program = clCreateProgramWithSource(lfcl_context,
            1, &source, &source_len, &cl_err);
    LF_CHECK_CL;
    LF_TRY(LFCL_compile(dirac_transport_program));

    // create kernels
    dirac_transport_filter_t = clCreateKernel(dirac_transport_program,
            "filter_t", &cl_err);
    LF_CHECK_CL;
    dirac_transport_filter_s = clCreateKernel(dirac_transport_program,
            "filter_s", &cl_err);
    LF_CHECK_CL;

    LF_ERROR_BLOCK;
    return ok;
}

extern int opencl_pillbox_transport_opencl_len;
extern unsigned char opencl_pillbox_transport_opencl[];
bool LFCL_build_pillbox_transport() {
    // TODO
    return true;

    LF_ERROR_START;

    size_t source_len = opencl_pillbox_transport_opencl_len;
    const char* source = (const char*)opencl_pillbox_transport_opencl;
    pillbox_transport_program = clCreateProgramWithSource(lfcl_context,
            1, &source, &source_len, &cl_err);
    LF_CHECK_CL;
    LF_TRY(LFCL_compile(pillbox_transport_program));

    LF_ERROR_BLOCK;
    return ok;
}

cl_kernel LFCL_dirac_transport_filter_t() {
    return dirac_transport_filter_t;
}

cl_kernel LFCL_dirac_transport_filter_s() {
    return dirac_transport_filter_t;
}

