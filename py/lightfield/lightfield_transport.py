import numpy as np
import ctypes as ct
import lightfield_base
lib = lightfield_base.lib
from lightfield_plane_geometry import PlaneGeometry
from lightfield_angular_plane import AngularPlane
from lightfield_optics import LFOptics, Optics

class Transport(ct.Structure):
    _field_ = [ \
            ('src_plane', ct.POINTER(PlaneGeometry)),
            ('dst_plane', ct.POINTER(PlaneGeometry)),
            ('angular_plane', ct.POINTER(AngularPlane)),
            ('src2plane_x', ct.POINTER(LFOptics)),
            ('src2plane_y', ct.POINTER(LFOptics)),
            ('dst2plane_x', ct.POINTER(LFOptics)),
            ('dst2plane_y', ct.POINTER(LFOptics)),
            ('src2dst_x', LFOptics),
            ('src2dst_y', LFOptics),
            ('dst2src_x', LFOptics),
            ('dst2src_y', LFOptics),
        ]

    def __init__(self, src, dst):
        self.src = src
        self.dst = dst

        lib.LFTransport_init(ct.pointer(self))
        lib.LFTransport_setup(ct.pointer(self),
                ct.pointer(self.src.plane_geometry),
                ct.pointer(self.dst.plane_geometry),
                ct.pointer(self.src.angular_plane),
                ct.pointer(self.src.optics_to_root.optics_x),
                ct.pointer(self.src.optics_to_root.optics_y),
                ct.pointer(self.dst.optics_to_root.optics_x),
                ct.pointer(self.dst.optics_to_root.optics_y))

    @property
    def src2dst(self):
        return Optics(self.src2dst_x, self.src2dst_y)

    @property
    def dst2src(self):
        return Optics(self.dst2src_x, self.dst2src_y)

    @property
    def src2plane(self):
        return Optics(self.src2plane_x.obj, src.src2plane_y.obj)

    @property
    def dst2plane(self):
        return Optics(self.dst2plane_x.obj, self.dst2plane_y.obj)

