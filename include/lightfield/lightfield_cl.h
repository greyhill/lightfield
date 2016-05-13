extern bool LFCL_simple_init();

extern cl_context LFCL_get_context();
extern cl_command_queue LFCL_get_queue();

// expand each entry of global_size until it is a multiple
// of local_size
extern void LFCL_fix_size(size_t dim, const size_t* local_size, 
        size_t* global_size);

// Transport kernels
extern cl_kernel LFCL_dirac_transport_filter_t();
extern cl_kernel LFCL_dirac_transport_filter_s();

