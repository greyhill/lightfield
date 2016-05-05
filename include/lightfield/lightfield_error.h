#define LF_TRY(predicate) \
    if(!(predicate)) { \
        fprintf(stderr, "Error occured at %s:%d\n", __FILE__, __LINE__); \
        goto err; \
    }
