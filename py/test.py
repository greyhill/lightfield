import lightfield
import numpy as np

print("Run this from the python directory")

# Rust implementation
print("Creating Rust handle")
rust = lightfield.Implementation("../target/release/liblightfield.so")

# C implementation
print("Creating C handle")
c = lightfield.Implementation("../liblightfield.so")

# Optical tests
rX = rust.OpticalX()
cX = c.OpticalX()

# identity
rX.to_identity()
cX.to_identity()
assert(rX == cX)

# translation
rX.to_translation(50.0)
cX.to_translation(50.0)
assert(rX == cX)

# to lens
rX.to_lens(12.0, 32.0, 5.0)
cX.to_lens(12.0, 32.0, 5.0)
assert(rX == cX)

# inverse
rXi = rX.invert()
cXi = cX.invert()
assert(rXi == cXi)

# composition
rX2 = rust.OpticalX().to_translation(500.0)
cX2 = c.OpticalX().to_translation(500.0)
rX3 = rX.compose(rX2)
cX3 = cX.compose(cX2)
assert(rX3 == cX3)

