# TODO 

## Rust code

### Features

- Opaque projection.  What would opaque backprojection look like?  Normal
    backprojection with a mask?  Actually, that sounds like a reconstruction
    algorithm...

### Refactors

- Convert `transport.rs` to using footprint calculations from `SplineKernel` objects

### Minor stuff / maybe

- Builds are broken on Apple's OpenCL implementation on my Macbook using Intel
    HD graphics.  Apple's OpenCL implementation is apparently kinda crappy, 
    and workarounds would be awkward, so maybe don't worry about this.

