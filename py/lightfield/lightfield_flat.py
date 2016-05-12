import ctypes as ct
import numpy as np
from lightfield_vec3 import Vec3
from lightfield_plane_geometry import PlaneGeometry

class FlatGeometry(ct.Structure):
    _fields_ = [ \
            ('n', Vec3),
            ('s', Vec3),
            ('t', Vec3),
            ('plane_geom', PlaneGeometry) ]
    def __init__(self,
            n, s, t,
            plane_geom):
        self.n = n
        self.s = s
        self.t = t
        self.plane_geom = plane_geom

    @property
    def ns(self):
        return self.plane_geom.ns

    @property
    def nt(self):
        return self.plane_geom.nt

    @property
    def shape(self):
        return (self.ns, self.nt)

    @property
    def zeros(self):
        return np.zeros(self.shape, dtype='float32', order='f')

    @property
    def ones(self):
        return np.ones(self.shape, dtype='float32', order='f')

