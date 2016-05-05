import ctypes as ct
import ctypes.util

libpath = ctypes.util.find_library('liblightfield')
if libpath is None:
    import os
    import os.path
    libpath = os.path.join(os.path.expanduser('~'), 'lib', 'liblightfield.so')
lib = ct.CDLL(libpath)

