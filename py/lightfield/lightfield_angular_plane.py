import numpy as np
import ctypes as ct
import lightfield_base

lib = lightfield_base.lib

lib.LFAngularPlane_setup.restype = ct.c_bool

class AngularPlane(ct.Structure):
    _fields_ = [ \
            ('du', ct.c_float),
            ('dv', ct.c_float),
            ('kind_raw', ct.c_int),
            ('coordinate_raw', ct.c_int),
            ('num_points', ct.c_size_t),
            ('u_points_raw', ct.POINTER(ct.c_float)),
            ('v_points_raw', ct.POINTER(ct.c_float)),
            ('w_points_raw', ct.POINTER(ct.c_float)) ]

    @staticmethod
    def type_str_to_int(s):
        if s == 'dirac':
            return 1
        elif s == 'box':
            return 2
        else:
            raise ValueError('Unknown AngularPlane type: "%s"' % s)

    @staticmethod
    def type_int_to_str(i):
        if i == 4:
            return 'dirac'
        elif i == 8:
            return 'box'
        else:
            raise ValueError('Unknown AngularPlane type id: %d' % i)

    @staticmethod
    def coordinate_str_to_int(s):
        if s == 'spatial':
            return 1
        elif s == 'angular':
            return 2
        else:
            raise ValueError('Unknown AngularPlane coordinate: "%s"' % s)

    @staticmethod
    def coordinate_int_to_str(i):
        if i == 4:
            return 'spatial'
        elif i == 8:
            return 'angular'
        else:
            raise ValueError('Unknown AngularPLane coordinate id: %d' % i)

    def __init__(self,
            u_points, v_points, w_points,
            du = 1.0, dv = 1.0,
            basis = 'dirac',
            coordinate = 'spatial'):
        u_points_ptr = np.asarray(u_points, dtype='float32')
        v_points_ptr = np.asarray(v_points, dtype='float32')
        w_points_ptr = np.asarray(w_points, dtype='float32')
        lib.LFAngularPlane_init(ct.pointer(self))
        if not lib.LFAngularPlane_setup(ct.pointer(self),
                ct.c_float(du),
                ct.c_float(dv),
                ct.c_int(AngularPlane.type_str_to_int(basis)),
                ct.c_int(AngularPlane.coordinate_str_to_int(coordinate)),
                ct.c_size_t(len(u_points)),
                ct.c_voidp(u_points_ptr.ctypes.data),
                ct.c_voidp(v_points_ptr.ctypes.data),
                ct.c_voidp(w_points_ptr.ctypes.data)):
            raise RuntimeError('error in LFAngularPlane_init')

    @property
    def u_points(self):
        return np.asarray(self.u_points_raw[:self.num_points])

    @property
    def v_points(self):
        return np.asarray(self.v_points_raw[:self.num_points])

    @property
    def w_points(self):
        return np.asarray(self.w_points_raw[:self.num_points])

    def get_kind(self):
        return AngularPlane.type_int_to_str(self.kind_raw)

    def set_kind(self, kind):
        self.kind_raw = AngularPlane.type_str_to_int(kind)

    kind = property(get_kind, set_kind)

    def get_coordinate(self):
        return AngularPLane.coordinate_int_to_str(self.coordinate_raw)

    def set_coordinate(self, coordinate):
        self.coordinate_raw = AngularPlane.coordinate_str_to_int(coordinate)

    coordinate = property(get_coordinate, set_coordinate)

