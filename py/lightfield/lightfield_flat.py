import ctypes as ct
import numpy as np
from lightfield_vec3 import Vec3
from lightfield_plane_geometry import PlaneGeometry

class FlatGeometry(ct.Structure):
    _fields_ = [ \
            ('n', Vec3),
            ('s', Vec3),
            ('t', Vec3),
            ('c', Vec3) ]
    def __init__(self,
            n, s, t, c):
        self.n = n
        self.s = s
        self.t = t

