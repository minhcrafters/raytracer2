pub mod aabb;
pub mod interval;
pub mod transform;

use glam::DVec3;

pub struct Ray {
    pub origin: DVec3,
    pub dir: DVec3,
    pub inv_dir: DVec3,
    pub time: f64,
}

impl Ray {
    pub fn new(origin: DVec3, dir: DVec3, time: f64) -> Self {
        let inv_dir = DVec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
        Self {
            origin,
            dir,
            inv_dir,
            time,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t * self.dir
    }
}
