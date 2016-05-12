#define LF_ERROR_START \
    cl_int cl_err = CL_SUCCESS; \
    bool ok = true;

#define LF_TRY(predicate) \
    if(!(predicate)) { \
        fprintf(stderr, "Error occured at %s:%d\n", __FILE__, __LINE__); \
        goto err; \
    }

#define LF_CHECK_CL \
    if(cl_err != CL_SUCCESS) { \
        fprintf(stderr, "CL error %d occurred at %s:%d\n", cl_err, __FILE__, __LINE__); \
        goto err; \
    }

#define LF_ERROR_BLOCK \
    if(0) { \
        err: \
        ok = false; \
    }
