num_dots = 256

template = '''
[[ellipsoids]]
xx = 1.0
xy = 0.0
xz = 0.0
yx = 0.0
yy = 1.0
yz = 0.0
zx = 0.0
zy = 0.0
zz = 1.0
xr = 0.2
yr = 0.2
zr = 0.2

xc = %f
yc = %f
zc = %f

value = 1.0

'''

import random
f = open("dots.toml", "w")
for _ in xrange(num_dots):
    xc = random.uniform(-112, 112)
    yc = random.uniform(-112, 112)
    zc = random.uniform(-185.5, 185.5)

    f.write(template % (xc, yc, zc))

