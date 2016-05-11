pub fn fmin(x: f32, y: f32) -> f32 {
    if x < y {
        x
    } else {
        y
    }
}

pub fn fmax(x: f32, y: f32) -> f32 {
    if x > y {
        x
    } else {
        y
    }
}

pub fn fclamp(minval: f32, maxval: f32, x: f32) -> f32 {
    fmax(fmin(maxval, x), minval)
}
