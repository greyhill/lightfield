import ctypes as ct

class Vec3(ct.Structure):
    _fields_ = [\
            ('x', ct.c_float),
            ('y', ct.c_float),
            ('z', ct.c_float) ]
    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z
