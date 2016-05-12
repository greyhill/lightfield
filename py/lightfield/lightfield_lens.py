import ctypes as ct

class Lens(ct.Structure):
    _fields_ = [\
            ('center_x', ct.c_float),
            ('center_y', ct.c_float),
            ('focal_length_x', ct.c_float),
            ('focal_length_y', ct.c_float),
            ('radius_x', ct.c_float),
            ('radius_y', ct.c_float)]
    def __init__(self, 
            focal_length,
            radius,
            center_x=0, center_y=0):
        self.focal_length_x = focal_length
        self.focal_length_y = focal_length
        self.radius_x = radius
        self.radius_y = radius
        self.center_x = center_x
        self.center_y = center_y

