import lightfield

print("Run this from the python directory")

# Rust implementaiton
rust = lightfield.Implementation("../target/release/liblightfield.so")

# C implementation
c = lightfield.Implementation("../liblightfield.so")

