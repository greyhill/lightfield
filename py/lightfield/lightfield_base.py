import ctypes as ct
import ctypes.util

libpath = ctypes.util.find_library('liblightfield')
if libpath is None:
    import os
    import os.path
    libpath = os.path.join(os.path.expanduser('~'), 'lib', 'liblightfield.so')
lib = ct.CDLL(libpath)

def simple_init():
    '''Do a simple OpenCL initialization
    
    '''
    lib.LFCL_simple_init.restype = ct.c_bool
    if not lib.LFCL_simple_init():
        raise RuntimeError('Error initializing OpenCL subsystem')

