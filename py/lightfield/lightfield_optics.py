import lightfield_base
import ctypes as ct

lib = lightfield_base.lib

class LFOptics(ct.Structure):
    '''One-dimensional affine optics.

    See Optics for a more useful 2D version.'''
    _fields_ = [\
            ('pp', ct.c_float),
            ('pa', ct.c_float),
            ('ap', ct.c_float),
            ('aa', ct.c_float),
            ('cp', ct.c_float),
            ('ca', ct.c_float) ]
    def __init__(self):
        lib.LFOptics_identity(ct.pointer(self))

    @staticmethod
    def translation(distance):
        tr = LFOptics()
        lib.LFOptics_translation(ct.pointer(tr), ct.c_float(distance))
        return tr

    @staticmethod
    def refraction(focal_length, center):
        tr = LFOptics()
        lib.LFOptics_refraction(ct.pointer(tr), ct.c_float(focal_length),
                ct.c_float(center))
        return tr

    def invert(self):
        tr = LFOptics()
        lib.LFOptics_invert(ct.pointer(self), ct.pointer(tr))
        return tr

    @property
    def inverse(self):
        return self.invert()

    @staticmethod
    def compose(lhs, rhs):
        tr = LFOptics()
        lib.LFOptics_compose(ct.pointer(lhs), ct.pointer(rhs), ct.pointer(tr))
        return tr

    def then(self, other):
        return LFOptics.compose(other, self)

    def before(self, other):
        return LFOptics.compose(self, other)

    def __str__(self):
        return "LFOptics: {pp: %f, pa: %f, ap: %f, aa: %f, cp: %f, ca: %f}" \
                % (self.pp, self.pa, self.ap, self.aa, self.cp, self.ca)

class Optics(object):
    '''Two-dimensional affine optics.

    Contains only two LFOptics fields: optics_x and optics_y.
    '''
    def __init__(self, optics_x=None, optics_y=None):
        self.optics_x = optics_x if optics_x is not None else LFOptics()
        self.optics_y = optics_y if optics_y is not None else LFOptics()

    def invert(self):
        return Optics(self.optics_x.invert(), self.optics_y.invert())

    @staticmethod
    def translation(self, distance):
        return Optics(LFOptics.translation(distance),
                      LFOptics.translation(distance))

    @staticmethod
    def refraction(self, focal_length, center_x, center_y):
        return Optics(LFOptics.refraction(focal_length, center_x),
                      LFOptics.refraction(focal_length, center_y))

    @property
    def inverse(self):
        return self.invert()

    @staticmethod
    def compose(lhs, rhs):
        return Optics(LFOptics.compose(lhs.optics_x, rhs.optics_x),
                      LFOptics.compose(lhs.optics_y, rhs.optics_y))

    def then(self, other):
        return Optics.compose(other, self)

    def before(self, other):
        return Optics.compose(self, other)

    def __str__(self):
        return "Optics: {x: %s, y: %s}" % (self.optics_x, self.optics_y)

