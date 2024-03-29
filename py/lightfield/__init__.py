import ctypes as ct
import numpy as np

class OpticalX(ct.Structure):
    '''Optical ray transformation'''
    _fields_ = [ \
            ('ss', ct.c_float),
            ('us', ct.c_float),
            ('su', ct.c_float),
            ('uu', ct.c_float),

            ('tt', ct.c_float),
            ('vt', ct.c_float),
            ('tv', ct.c_float),
            ('vv', ct.c_float),

            ('s', ct.c_float),
            ('t', ct.c_float),
            ('u', ct.c_float),
            ('v', ct.c_float) ]

    def __init__(self, impl):
        '''Create a new optical transformation (using the given implementation).'''
        self._lib = impl.lib
        self._lib.LFOpticalX_identity(ct.pointer(self))

    def compose(self, other):
        '''Returns the composition of this formation and the given one;
        mathematically `this * other`.'''
        to_return = OpticalX(self._lib)
        self._lib.LFOpticalX_compose(\
                ct.pointer(self),
                ct.pointer(other),
                ct.pointer(to_return))
        return to_return

    def to_identity(self):
        '''Turns this transformation into an identity transformation.'''
        self._lib.LFOpticalX_identity(ct.pointer(self))
        return self

    def to_translation(self, distance):
        '''Turns this transformation into translation through free
        space by a given distance.'''
        self._lib.LFOpticalX_translation(ct.pointer(self),
                ct.c_float(distance))
        return self

    def to_lens(self, center_x, center_y, focal_length):
        '''Turns this transformation into refraction by a lens.'''
        self._lib.LFOpticalX_lens(ct.pointer(self),
                ct.c_float(center_x), ct.c_float(center_y),
                ct.c_float(focal_length))
        return self

    def invert(self):
        '''Returns the inverse of this transformation.'''
        to_return = OpticalX(self._lib)
        self._lib.LFOpticalX_invert(ct.pointer(self),
                ct.pointer(to_return))
        return to_return
        
    def then(self, other):
        '''The same as `other.compose(self)`.'''
        return other.compose(self)
        
    def __mul__(self, other):
        return self.compose(other)

    def __eq__(self, other):
        return self.ss == other.ss and \
               self.su == other.su and \
               self.us == other.us and \
               self.uu == other.uu and \
               \
               self.tt == other.tt and \
               self.tv == other.tv and \
               self.vt == other.vt and \
               self.vv == other.vv and \
               \
               self.s == other.s and \
               self.t == other.t and \
               self.u == other.u and \
               self.v == other.v

class ImageGeometry(ct.Structure):
    '''Structure representing a 2D image or a view of a light field.'''
    _fields_ = [ 
            ('ns', ct.c_size_t), 
            ('nt', ct.c_size_t),

            ('ds', ct.c_float),
            ('dt', ct.c_float),

            ('offset_s', ct.c_float),
            ('offset_t', ct.c_float) ]

class AngularPlane(ct.Structure):
    '''Structure representing the angular plane of a light transport system.'''
    _fields_ = [
            ('ds', ct.c_float),
            ('dt', ct.c_float),

            ('basis_enum', ct.c_int),

            ('num_points', ct.c_size_t),
            ('points_s', ct.POINTER(ct.c_float)),
            ('points_t', ct.POINTER(ct.c_float)),
            ('points_w', ct.POINTER(ct.c_float)) ]

    def __init__(self, 
            ds, dt, basis_str,
            s_points, t_points, w_points):
        self._s_points = np.asarray(s_points, dtype='float32')
        self._t_points = np.asarray(t_points, dtype='float32')
        self._w_points = np.asarray(w_points, dtype='float32')

        basis_enum = { 'dirac': 1, 'pillbox': 0 }
        self.basis_enum = basis_enum[basis_str]

        self.ds = ds
        self.dt = dt

        self.points_s = ct.c_voidp(self._s_points.ctypes.data)
        self.points_t = ct.c_voidp(self._t_points.ctypes.data)
        self.points_w = ct.c_voidp(self._w_points.ctypes.data)
        
    def get_basis(self):
        basis_conv = [ 'pillbox', 'dirac' ]
        return basis_conv[self.basis_enum]
        
    def set_basis(self, val):
        basis_conv = { 'pillbox': 0, 'dirac': 1 }
        self.basis_enum = basis_conv[val]
        
    basis = property(get_basis, set_basis)

class LFGeometry(ct.Structure):
    '''Structure representing the geometry of a light field.  Contains
    the geometry of each view (`geom`), the angular plane (`plane`),
    and the optical transformation to the angular plane (`to_plane`).'''
    _fields_ = [ 
            ('geom', ImageGeometry),
            ('plane', AngularPlane),
            ('to_plane', OpticalX)
        ]

class Transport(object):
    '''Fundamental light field transport operation.'''
    def __init__(self, src, dst, impl):
        self.src = src
        self.dst = dst
        self.ptr = None
        self.impl = impl

        self.ptr = self.impl.lib.LFTransport_new(
                ct.pointer(src), ct.pointer(dst),
                impl.env)
        if self.ptr is None:
            raise RuntimeError("Error creating Transport object")

    def __del__(self):
        if self.ptr is not None:
            self.impl.lib.LFTransport_del(self.ptr)

    def forw_view(self, src, angle):
        '''Forward-transport the given view from the source light field to the 
        destination.'''
        tr = np.zeros((self.dst.ns, self.dst.nt), dtype='float32', order='f')
        src = np.asarray(src, dtype='float32', order='f')
        if not self.impl.lib.LFTransport_forw_view(self.ptr,
                ct.c_voidp(src.ctypes.data),
                ct.c_voidp(tr.ctypes.data),
                ct.c_size_t(angle)):
            raise RuntimeError('Error in LFTransport_forw_view')
        else:
            return tr

    def back_view(self, dst, angle):
        '''Adjoint of the forward projection operation.''' 
        tr = np.zeros((self.src.ns, self.src.nt), dtype='float32', order='f')
        dst = np.asarray(dst, dtype='float32', order='f')
        if not self.impl.lib.LFTransport_back_view(self.ptr,
                ct.c_voidp(dst.ctypes.data),
                ct.c_voidp(tr.ctypes.data),
                ct.c_size_t(angle)):
            raise RuntimeError('Error in LFTransport_back_view')
        else:
            return tr

class Implementation(object):
    '''Wrapper for the native component of the `lightfield` package.'''
    def __init__(self, path):
        '''Loads the implementation in the given shared library'''
        self.env = None
        self.lib = ct.CDLL(path)

        self._setup_calls()
        self._setup_environment()

    def __del__(self):
        if self.env is not None:
            self.lib.LFEnvironment_del(self.env)

    def _setup_calls(self):
        self.lib.LFEnvironment_new.restype = ct.c_voidp

        self.lib.LFTransport_new.restype = ct.c_voidp
        self.lib.LFTransport_forw_view.restype = ct.c_bool
        self.lib.LFTransport_back_view.restype = ct.c_bool

    def _setup_environment(self):
        env = self.lib.LFEnvironment_new()
        if env is None:
            raise RuntimeError("Error creating environment")
        self.env = env

    def OpticalX(self):
        return OpticalX(self.lib)

    def Transport(self, src, dst):
        return Transport(src, dst, self)
