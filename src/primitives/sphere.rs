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
    pub radius: f64,
    pub material: Option<Arc<dyn Material>>,
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let oc = self.center - r.origin;
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
        let normal = (point - self.center) / self.radius;

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
