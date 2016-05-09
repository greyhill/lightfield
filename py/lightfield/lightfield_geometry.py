import numpy as np
import ctypes as ct
import lightfield_base
lib = lightfield_base.lib
from lightfield_transport import Transport

lib.LFPlaneGeometry_ws.restype = ct.c_float
lib.LFPlaneGeometry_wt.restype = ct.c_float

class LightFieldGeometry(object):
    def __init__(self, 
            plane_geometry,
            angular_plane,
            optics_to_root,
            smin = None,
            smax = None,
            tmin = None,
            tmax = None):
        self.plane_geometry = plane_geometry
        self.angular_plane = angular_plane
        self.optics_to_root = optics_to_root
        self.smin = smin if smin is not None else 0
        self.smax = smax if smax is not None else self.plane_geometry.ns
        self.tmin = tmin if tmin is not None else 0
        self.tmax = tmax if tmax is not None else self.plane_geometry.nt

    @property
    def ns(self):
        return self.smax - self.smin

    @property
    def nt(self):
        return self.tmax - self.tmin

    @property
    def shape(self):
        return (self.plane_geometry.ns, self.plane_geometry.nt)

    @property
    def zeros(self):
        return np.zeros(self.shape, dtype='float32', order='f')

    @property
    def ones(self):
        return np.ones(self.shape, dtype='float32', order='f')

    def transport_to(self, other):
        return Transport(self, other)

