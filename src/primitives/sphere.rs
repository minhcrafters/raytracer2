use std::sync::Arc;

use glam::DVec3;

use crate::{
    material::Material,
    ray::{
        Ray,
        hittable::{HitRecord, Hittable},
        interval::Interval,
    },
};

pub struct Sphere {
    pub center: DVec3,
    pub center_vec: DVec3,
    pub is_moving: bool,
    pub radius: f64,
    pub material: Option<Arc<dyn Material>>,
}

impl Sphere {
    pub fn stationary(center: DVec3, radius: f64, material: Option<Arc<dyn Material>>) -> Self {
        Self {
            center,
            center_vec: DVec3::ZERO,
            is_moving: false,
            radius,
            material,
        }
    }

    pub fn moving(
        center1: DVec3,
        center2: DVec3,
        radius: f64,
        material: Option<Arc<dyn Material>>,
    ) -> Self {
        Self {
            center: center1,
            center_vec: center2 - center1,
            is_moving: true,
            radius,
            material,
        }
    }

    pub fn get_center(&self, time: f64) -> DVec3 {
        if self.is_moving {
            self.center + time * self.center_vec
        } else {
            self.center
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let current_center = self.get_center(r.time);
        let oc = current_center - r.origin;
        let a = r.dir.length_squared();
        let h = r.dir.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (h - sqrtd) / a;
        if !interval.surrounds(root) {
            root = (h + sqrtd) / a;
            if !interval.surrounds(root) {
                return None;
            }
        }

        let point = r.at(root);
        let normal = (point - current_center) / self.radius;

        let mut rec = HitRecord {
            point,
            normal,
            material: self.material.clone(),
            t: root,
            front_face: false,
        };

        rec.set_face_normal(r, normal);

        Some(rec)
    }
}
