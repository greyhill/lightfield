import numpy as np
import ctypes as ct
import lightfield_base
lib = lightfield_base.lib

lib.LFPlaneGeometry_ws.restype = ct.c_float
lib.LFPlaneGeometry_wt.restype = ct.c_float
lib.LFLixel_volume.restype = ct.c_float

class LightFieldGeometry(object):
    def __init__(self, 
            plane_geometry,
            angular_plane,
            optics_to_root):
        self.plane_geometry = plane_geometry
        self.angular_plane = angular_plane
        self.optics_to_root = optics_to_root

    @property
    def ns(self):
        return self.plane_geometry.ns

    @property
    def nt(self):
        return self.plane_geometry.nt

    @property
    def shape(self):
        return (self.ns, self.nt)

    @property
    def zeros(self):
        return np.zeros(self.shape, dtype='float32', order='f')

    @property
    def lixel_volume(self):
        return lib.LFLixel_volume(ct.pointer(self.plane_geometry),
                ct.pointer(self.angular_plane),
                ct.pointer(self.optics_to_root.optics_x),
                ct.pointer(self.optics_to_root.optics_y))

    @property
    def ones(self):
        return np.ones(self.shape, dtype='float32', order='f')

