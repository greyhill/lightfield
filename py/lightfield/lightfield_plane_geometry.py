import numpy as np
import lightfield_base
import ctypes as ct
lib = lightfield_base.lib

lib.LFPlaneGeometry_ws.restype = ct.c_float
lib.LFPlaneGeometry_wt.restype = ct.c_float

class PlaneGeometry(ct.Structure):
    _fields_ = [\
            ('ns', ct.c_size_t),
            ('nt', ct.c_size_t),
            ('ds', ct.c_float),
            ('dt', ct.c_float),
            ('offset_s', ct.c_float),
            ('offset_t', ct.c_float) ]

    def __init__(self, ns, nt,
            ds = 1.0, dt = 1.0,
            offset_s = 0.0, offset_t = 0.0):
        self.ns = ns
        self.nt = nt
        self.ds = ds
        self.dt = dt
        self.offset_s = offset_s
        self.offset_t = offset_t

    @property
    def ws(self):
        return lib.LFPlaneGeometry_ws(ct.pointer(self))

    @property
    def wt(self):
        return lib.LFPlaneGeometry.wt(ct.pointer(self))

    @property
    def shape(self):
        return (self.ns, self.nt)

    @property
    def zeros(self):
        return np.zeros(self.shape, dtype='float32', order='f')

    @property
    def ones(self):
        return np.ones(self.shape, dtype='float32', order='f')

