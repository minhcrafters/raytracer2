pub mod instance;
pub mod model;
pub mod quad;
pub mod sphere;
pub mod triangle;

use glam::DVec3;
use std::sync::Arc;

use crate::{material::Material, ray::interval::Interval};

use super::{ray::Ray, ray::aabb::Aabb};

pub struct HitRecord {
    pub point: DVec3,
    pub normal: DVec3,
    pub material: Option<Arc<dyn Material>>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
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

pub trait Hittable: Send + Sync {
    fn pdf_value(&self, _origin: glam::DVec3, _direction: glam::DVec3) -> f64 {
        0.0
    }

    fn random(&self, _origin: glam::DVec3) -> glam::DVec3 {
        glam::DVec3::new(1.0, 0.0, 0.0)
    }
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord>;
    fn bounding_box(&self) -> Aabb;
}

pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>,
    pub bbox: Aabb,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: Aabb::default(),
        }
    }

    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        self.bbox = Aabb::from_aabbs(&self.bbox, &object.bounding_box());
        self.objects.push(object);
    }

    pub fn append_list(&mut self, list: &HittableList) {
        self.bbox = Aabb::from_aabbs(&self.bbox, &list.bbox);
        self.objects.extend_from_slice(&list.objects);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.bbox = Aabb::default();
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

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn pdf_value(&self, origin: glam::DVec3, direction: glam::DVec3) -> f64 {
        if self.objects.is_empty() {
            return 0.0;
        }
        let weight = 1.0 / self.objects.len() as f64;
        let mut sum = 0.0;
        for object in &self.objects {
            sum += weight * object.pdf_value(origin, direction);
        }
        sum
    }

    fn random(&self, origin: glam::DVec3) -> glam::DVec3 {
        if self.objects.is_empty() {
            return glam::DVec3::new(1.0, 0.0, 0.0);
        }
        let int_size = self.objects.len();
        let rand_idx = (crate::utils::random_f64() * int_size as f64).floor() as usize;
        let rand_idx = rand_idx.clamp(0, int_size.saturating_sub(1));
        self.objects[rand_idx].random(origin)
    }
}
