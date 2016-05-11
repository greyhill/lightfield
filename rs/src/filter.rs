use float_util::*;

/// Trait for a Toeplitz-like filter
pub trait Filter {
    fn jmin(self: &Self, i: usize, jmax: usize) -> usize;
    fn jmax(self: &Self, i: usize, jmax: usize) -> usize;
    /// Compute the (i,j)th entry of the operator
    fn evaluate(self: &Self, i: usize, j: usize) -> f32;
}

/// Rectangular/box filter
pub struct Rect {
    height: f32,
    i_scale: f32,
    t0: f32,
    t1: f32,
}

impl Filter for Rect {
    fn jmin(self: &Self, i: usize, jmax: usize) -> usize {
        let t0 = self.t0 + (i as f32)*self.i_scale;
        fclamp(0f32, jmax as f32, t0) as usize
    }

    fn jmax(self: &Self, i: usize, jmax: usize) -> usize {
        let t1 = self.t1 + (i as f32)*self.i_scale;
        fclamp(0f32, jmax as f32, t1) as usize
    }

    fn evaluate(self: &Self, i: usize, j: usize) -> f32 {
        let t0 = self.t0 + (i as f32)*self.i_scale;
        let t1 = self.t1 + (i as f32)*self.i_scale;

        let l = fclamp(t0, t1, j as f32);
        let r = fclamp(t0, t1, j as f32 + 1f32);
        (r - l)*self.height
    }
}

/// Trapezoidal filter
pub struct Trap {
    height: f32,
    i_scale: f32,
    t0: f32,
    t1: f32,
    t2: f32, 
    t3: f32,
}

impl Filter for Trap {
    fn jmin(self: &Self, i: usize, jmax: usize) -> usize {
        let t0 = self.t0 + (i as f32)*self.i_scale;
        fclamp(0f32, jmax as f32, t0) as usize
    }

    fn jmax(self: &Self, i: usize, jmax: usize) -> usize {
        let t3 = self.t1 + (i as f32)*self.i_scale;
        fclamp(0f32, jmax as f32, t3) as usize
    }

    fn evaluate(self: &Self, i: usize, j: usize) -> f32 {
        let t0 = self.t0 + (i as f32)*self.i_scale;
        let t1 = self.t1 + (i as f32)*self.i_scale;
        let t2 = self.t2 + (i as f32)*self.i_scale;
        let t3 = self.t3 + (i as f32)*self.i_scale;
        let mut accum = 0f32;

        let mut l = fclamp(t0, t1, j as f32);
        let mut r = fclamp(t0, t1, j as f32 + 1f32);
        accum += ((r - t0).powi(2) - (l - t0).powi(2))/(2f32 * (t1 - t0));

        l = fclamp(t1, t2, j as f32);
        r = fclamp(t1, t2, j as f32 + 1f32);
        accum += r - l;

        l = fclamp(t2, t3, j as f32);
        r = fclamp(t2, t3, j as f32 + 1f32);
        accum += ((l - t3).powi(2) - (r - t3).powi(2))/(2f32 * (t3 - t2));

        accum * self.height
    }
}

