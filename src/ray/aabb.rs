use glam::DVec3;

use super::{interval::Interval, transform::Transform, Ray};

#[derive(Clone, Copy)]
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Aabb {
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    pub fn from_points(a: DVec3, b: DVec3) -> Self {
        Self {
            x: Interval::new(a.x.min(b.x), a.x.max(b.x)),
            y: Interval::new(a.y.min(b.y), a.y.max(b.y)),
            z: Interval::new(a.z.min(b.z), a.z.max(b.z)),
        }
    }

    pub fn from_aabbs(box0: &Aabb, box1: &Aabb) -> Self {
        Self {
            x: Interval::new(box0.x.min.min(box1.x.min), box0.x.max.max(box1.x.max)),
            y: Interval::new(box0.y.min.min(box1.y.min), box0.y.max.max(box1.y.max)),
            z: Interval::new(box0.z.min.min(box1.z.min), box0.z.max.max(box1.z.max)),
        }
    }

    pub fn axis(&self, n: usize) -> &Interval {
        match n {
            1 => &self.y,
            2 => &self.z,
            _ => &self.x,
        }
    }

    pub fn hit(&self, r: &Ray, mut ray_t: Interval) -> bool {
        for a in 0..3 {
            let inv_d = r.inv_dir[a];
            let orig = r.origin[a];

            let mut t0 = (self.axis(a).min - orig) * inv_d;
            let mut t1 = (self.axis(a).max - orig) * inv_d;

            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            if t0 > ray_t.min {
                ray_t.min = t0;
            }
            if t1 < ray_t.max {
                ray_t.max = t1;
            }

            if ray_t.max <= ray_t.min {
                return false;
            }
        }
        true
    }

    pub fn transform(&self, transform: &Transform) -> Self {
        let mut min = DVec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = DVec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = if i == 0 { self.x.min } else { self.x.max };
                    let y = if j == 0 { self.y.min } else { self.y.max };
                    let z = if k == 0 { self.z.min } else { self.z.max };

                    let tester = DVec3::new(x, y, z);
                    let transformed = transform.transform_point(tester);

                    for c in 0..3 {
                        min[c] = f64::min(min[c], transformed[c]);
                        max[c] = f64::max(max[c], transformed[c]);
                    }
                }
            }
        }

        Self::from_points(min, max)
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            x: Interval::new(f64::INFINITY, f64::NEG_INFINITY),
            y: Interval::new(f64::INFINITY, f64::NEG_INFINITY),
            z: Interval::new(f64::INFINITY, f64::NEG_INFINITY),
        }
    }
}
