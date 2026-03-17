use glam::DVec3;
use std::sync::Arc;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::{Ray, aabb::Aabb, interval::Interval},
};

pub struct Triangle {
    pub q: DVec3,
    pub u: DVec3,
    pub v: DVec3,
    pub material: Option<Arc<dyn Material>>,
    bbox: Aabb,
    normal: DVec3,
    d: f64,
    w: DVec3,
}

impl Triangle {
    pub fn new(q: DVec3, u: DVec3, v: DVec3, material: Option<Arc<dyn Material>>) -> Self {
        let n = u.cross(v);
        let normal = n.normalize();
        let d = normal.dot(q);
        let w = n / n.length_squared();

        let bbox_diagonal1 = Aabb::from_points(q, q + u);
        let bbox_diagonal2 = Aabb::from_points(q, q + v);
        let mut bbox = Aabb::from_aabbs(&bbox_diagonal1, &bbox_diagonal2);

        let delta = 0.0001;
        if bbox.x.size() < delta {
            bbox.x = Interval::new(bbox.x.min - delta / 2.0, bbox.x.max + delta / 2.0);
        }
        if bbox.y.size() < delta {
            bbox.y = Interval::new(bbox.y.min - delta / 2.0, bbox.y.max + delta / 2.0);
        }
        if bbox.z.size() < delta {
            bbox.z = Interval::new(bbox.z.min - delta / 2.0, bbox.z.max + delta / 2.0);
        }

        Self {
            q,
            u,
            v,
            material,
            bbox,
            normal,
            d,
            w,
        }
    }
}

impl Hittable for Triangle {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let denom = self.normal.dot(r.dir);

        // No hit if the ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return None;
        }

        // Return false if the hit point parameter t is outside the ray interval.
        let t = (self.d - self.normal.dot(r.origin)) / denom;
        if !interval.contains(t) {
            return None;
        }

        // Determine the hit point lies within the planar shape using its principal 2D axes.
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = self.w.dot(planar_hitpt_vector.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hitpt_vector));

        // For a triangle, alpha + beta must be less than or equal to 1.0
        if alpha < 0.0 || beta < 0.0 || (alpha + beta) > 1.0 {
            return None;
        }

        let mut rec = HitRecord {
            point: intersection,
            normal: self.normal,
            material: self.material.clone(),
            t,
            u: alpha,
            v: beta,
            front_face: false,
        };

        rec.set_face_normal(r, self.normal);

        Some(rec)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn pdf_value(&self, origin: DVec3, direction: DVec3) -> f64 {
        let ray = Ray::new(origin, direction, 0.0);
        if let Some(rec) = self.hit(&ray, &Interval::new(0.001, f64::INFINITY)) {
            let distance_squared = rec.t * rec.t * direction.length_squared();
            let cosine = (direction.dot(rec.normal) / direction.length()).abs();

            let area = 0.5 * self.u.cross(self.v).length();
            distance_squared / (cosine * area)
        } else {
            0.0
        }
    }

    fn random(&self, origin: DVec3) -> DVec3 {
        let mut r1 = crate::utils::random_f64();
        let mut r2 = crate::utils::random_f64();
        if r1 + r2 > 1.0 {
            r1 = 1.0 - r1;
            r2 = 1.0 - r2;
        }
        let p = self.q + (r1 * self.u) + (r2 * self.v);
        p - origin
    }
}
