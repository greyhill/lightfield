// vim: filetype=opencl
//
// n.b., this file is customarily built with transport_dirac_f32.opencl,
// image_geom_f32.opencl, light_volume_f32.opencl, optics_f32.opencl,
// spline_kernel_f32.opencl

kernel void volume_forw_t(
        LightVolume volume_geom,
        ImageGeometry slice_geom,
        ImageGeometry dst_geom,
        
        global struct Optics* dst_to_slice,
        global struct RectSplineKernel* splines_t,
        
        global const float* volume,
        global float* tmp) {
}

kernel void volume_forw_s(
        LightVolume volume_geom,
        ImageGeometry slice_geom,
        ImageGeometry dst_geom,
        
        global struct Optics* dst_to_slice,
        global struct RectSplineKernel* spline_s,
        
        global const float* tmp,
        global float* dst) {
}

kernel void volume_back_t() {
}

kernel void volume_back_s() {
}

