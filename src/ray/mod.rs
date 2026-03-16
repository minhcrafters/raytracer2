pub mod hittable;
pub mod interval;

use glam::DVec3;

pub struct Ray {
    pub origin: DVec3,
    pub dir: DVec3,
    pub time: f64,
}

impl Ray {
    pub fn new(origin: DVec3, dir: DVec3, time: f64) -> Self {
        Self { origin, dir, time }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t * self.dir
    }
}
