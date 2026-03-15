use glam::DVec3;

use std::sync::Arc;

use crate::{material::Material, ray::interval::Interval};

use super::Ray;

pub struct HitRecord {
    pub point: DVec3,
    pub normal: DVec3,
    pub material: Option<Arc<dyn Material>>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: DVec3) {
        self.front_face = ray.dir.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord>;
}

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

        Some(HitRecord {
            point,
            normal,
            material: self.material.clone(),
            t: root,
            front_face: false,
        })
    }
}

pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let mut hit_record = None;
        let mut closest_so_far = interval.max;
        for object in &self.objects {
            if let Some(temp_rec) = object.hit(r, &Interval::new(interval.min, closest_so_far)) {
                closest_so_far = temp_rec.t;
                hit_record = Some(temp_rec);
            }
        }

        hit_record
    }
}
