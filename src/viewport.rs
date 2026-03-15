pub struct Viewport {
    pub width: f64,
    pub height: f64,
    pub focal_length: f64,
}

impl Viewport {
    pub fn new(width: f64, height: f64, focal_length: f64) -> Self {
        Self {
            width,
            height,
            focal_length,
        }
    }

    pub fn get_delta_uv(&self, x: f64, y: f64) -> (f64, f64) {
        let u = x / self.width;
        let v = y / self.height;
        (u, v)
    }
}
