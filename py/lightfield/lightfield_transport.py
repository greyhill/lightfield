from lightfield_base import lib
from lightfield_angular_plane import AngularPlane
from lightfield_plane_geometry import PlaneGeometry
from lightfield_optics import LFOptics, Optics
import ctypes as ct

lib.LFTransport_init.restype = ct.c_bool
lib.LFTransport_del.restype = ct.c_bool
lib.LFTransport_setup.restype = ct.c_bool
lib.LFTransport_tmp_size.restype = ct.c_size_t
lib.LFTransport_compute.restype = ct.c_bool

class Transport(ct.Structure):
    _fields_ = [\
            ('angular_plane', ct.POINTER(AngularPlane)),
            ('src_plane', ct.POINTER(PlaneGeometry)),
            ('src_to_root_s', ct.POINTER(LFOptics)),
            ('src_to_root_t', ct.POINTER(LFOptics)),
            ('dst_plane', ct.POINTER(PlaneGeometry)),
            ('dst_to_root_s', ct.POINTER(LFOptics)),
            ('dst_to_root_t', ct.POINTER(LFOptics)),
            ('scale', ct.c_float),
            ('src_to_dst_s', LFOptics),
            ('src_to_dst_t', LFOptics) ]
    def __init__(self, src, dst, scale = 1.0):
        self.src = src
        self.dst = dst
        lib.LFTransport_init(ct.pointer(self))
        if not lib.LFTransport_setup(ct.pointer(self),
                ct.pointer(self.src.angular_plane),
                ct.pointer(self.src.plane_geometry),
                ct.pointer(self.src.optics_to_root.optics_x),
                ct.pointer(self.src.optics_to_root.optics_y),
                ct.pointer(self.dst.plane_geometry),
                ct.pointer(self.dst.optics_to_root.optics_x),
                ct.pointer(self.dst.optics_to_root.optics_y),
                ct.c_float(scale)):
            raise RuntimeError('Error in LFTransport_setup')

    def __del__(self):
        lib.LFTransport_del(ct.pointer(self))

    @property
    def tmp_size(self):
        return lib.LFTransport_tmp_size(ct.pointer(self))

