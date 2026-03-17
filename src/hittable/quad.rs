use glam::DVec3;
use std::sync::Arc;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::{Ray, aabb::Aabb, interval::Interval, transform::Transform},
};

pub struct Quad {
    pub q: DVec3,
    pub u: DVec3,
    pub v: DVec3,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
    bbox: Aabb,
    normal: DVec3,
    d: f64,
    w: DVec3,
}

impl Quad {
    pub fn new(q: DVec3, u: DVec3, v: DVec3, material: Option<Arc<dyn Material>>) -> Self {
        let n = u.cross(v);
        let normal = n.normalize();
        let d = normal.dot(q);
        let w = n / n.length_squared();

        let bbox_diagonal1 = Aabb::from_points(q, q + u + v);
        let bbox_diagonal2 = Aabb::from_points(q + u, q + v);
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
            transform: Transform::default(),
            bbox,
            normal,
            d,
            w,
        }
    }

    pub fn cube(center: DVec3, size: f64, material: Option<Arc<dyn Material>>) -> Vec<Self> {
        let half_size = size / 2.0;
        let u = DVec3::new(size, 0.0, 0.0);
        let v = DVec3::new(0.0, size, 0.0);
        let w = DVec3::new(0.0, 0.0, size);

        vec![
            Quad::new(center - half_size * (u + v + w), u, v, material.clone()), // Front face
            Quad::new(center - half_size * (u + v - w), u, -v, material.clone()), // Back face
            Quad::new(center - half_size * (u - v + w), u, w, material.clone()), // Top face
            Quad::new(center - half_size * (u - v - w), u, -w, material.clone()), // Bottom face
            Quad::new(center - half_size * (-u + v + w), v, w, material.clone()), // Left face
            Quad::new(center - half_size * (-u + v - w), v, -w, material.clone()), // Right face
        ]
    }
}

impl Hittable for Quad {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let ray_obj = Ray::new(
            self.transform.inverse_transform_point(r.origin),
            self.transform.inverse_transform_vector(r.dir),
            r.time,
        );

        let denom = self.normal.dot(ray_obj.dir);

        // No hit if the ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return None;
        }

        // Return false if the hit point parameter t is outside the ray interval.
        let t = (self.d - self.normal.dot(ray_obj.origin)) / denom;
        if !interval.contains(t) {
            return None;
        }

        // Determine the hit point lies within the planar shape using its principal 2D axes.
        let intersection_obj = ray_obj.at(t);
        let planar_hitpt_vector = intersection_obj - self.q;
        let alpha = self.w.dot(planar_hitpt_vector.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hitpt_vector));

        if alpha < 0.0 || alpha > 1.0 || beta < 0.0 || beta > 1.0 {
            return None;
        }

        let normal_world = self.transform.transform_normal(self.normal);

        let mut rec = HitRecord {
            point: self.transform.transform_point(intersection_obj),
            normal: normal_world,
            material: self.material.clone(),
            t,
            u: alpha,
            v: beta,
            front_face: false,
        };

        rec.set_face_normal(r, normal_world);

        Some(rec)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox.transform(&self.transform)
    }

    fn pdf_value(&self, origin: DVec3, direction: DVec3) -> f64 {
        let ray = Ray::new(origin, direction, 0.0);
        if let Some(rec) = self.hit(&ray, &Interval::new(0.001, f64::INFINITY)) {
            let distance_squared = rec.t * rec.t * direction.length_squared();
            let cosine = (direction.dot(rec.normal) / direction.length()).abs();

            let world_u = self.transform.transform_vector(self.u);
            let world_v = self.transform.transform_vector(self.v);
            let area = world_u.cross(world_v).length();

            distance_squared / (cosine * area)
        } else {
            0.0
        }
    }

    fn random(&self, origin: DVec3) -> DVec3 {
        let p_obj =
            self.q + (crate::utils::random_f64() * self.u) + (crate::utils::random_f64() * self.v);
        let p_world = self.transform.transform_point(p_obj);
        p_world - origin
    }
}
