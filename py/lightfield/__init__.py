import ctypes as ct

class OpticalX(ct.Structure):
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

    def __init__(self, lib):
        self._lib = lib
        self._lib.LFOpticalX_identity(ct.pointer(self))

    def compose(self, other):
        to_return = OpticalX(self._lib)
        self._lib.LFOpticalX_compose(\
                ct.pointer(self),
                ct.pointer(other),
                ct.pointer(to_return))
        return to_return

    def to_identity(self):
        self._lib.LFOpticalX_identity(ct.pointer(self))
        return self

    def to_translation(self, distance):
        self._lib.LFOpticalX_translation(ct.pointer(self),
                ct.c_float(distance))
        return self

    def to_lens(self, center_x, center_y, focal_length):
        self._lib.LFOpticalX_lens(ct.pointer(self),
                ct.c_float(center_x), ct.c_float(center_y),
                ct.c_float(focal_length))
        return self

    def invert(self):
        to_return = OpticalX(self._lib)
        self._lib.LFOpticalX_invert(ct.pointer(self),
                ct.pointer(to_return))
        return to_return

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

class Implementation(object):
    def __init__(self, path):
        self.env = None
        self.lib = ct.CDLL(path)

        self._setup_calls()
        self._setup_environment()

    def __del__(self):
        if self.env is not None:
            self.lib.LFEnvironment_del(self.env)

    def _setup_calls(self):
        self.lib.LFEnvironment_new.restype = ct.c_voidp

    def _setup_environment(self):
        env = self.lib.LFEnvironment_new()
        if env is None:
            raise RuntimeError("Error creating environment")
        self.env = env

    def OpticalX(self):
        return OpticalX(self.lib)

