extern bool LFCL_simple_init();

extern cl_context LFCL_get_context();
extern cl_command_queue LFCL_get_queue();

// Transport kernels
extern cl_kernel LFCL_dirac_transport_filter_t();
extern cl_kernel LFCL_dirac_transport_filter_s();

