use std::sync::Arc;

use glam::DVec3;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::{Ray, aabb::Aabb, interval::Interval, transform::Transform},
};

use super::quad::Quad;

pub struct Cuboid {
    faces: Vec<Quad>,
    pub transform: Transform,
    bbox: Aabb,
}

impl Cuboid {
    pub fn new(a: DVec3, b: DVec3, material: Option<Arc<dyn Material>>) -> Self {
        let min = DVec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
        let max = DVec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

        let dx = DVec3::new(max.x - min.x, 0.0, 0.0);
        let dy = DVec3::new(0.0, max.y - min.y, 0.0);
        let dz = DVec3::new(0.0, 0.0, max.z - min.z);

        let mut faces = Vec::with_capacity(6);

        faces.push(Quad::from_points(
            DVec3::new(min.x, min.y, max.z),
            dx,
            dy,
            material.clone(),
        ));
        faces.push(Quad::from_points(
            DVec3::new(max.x, min.y, min.z),
            -dx,
            dy,
            material.clone(),
        ));
        faces.push(Quad::from_points(
            DVec3::new(min.x, min.y, min.z),
            dz,
            dy,
            material.clone(),
        ));
        faces.push(Quad::from_points(
            DVec3::new(max.x, min.y, max.z),
            -dz,
            dy,
            material.clone(),
        ));
        faces.push(Quad::from_points(
            DVec3::new(min.x, max.y, max.z),
            dx,
            -dz,
            material.clone(),
        ));
        faces.push(Quad::from_points(
            DVec3::new(min.x, min.y, min.z),
            dx,
            dz,
            material.clone(),
        ));

        let mut bbox = Aabb::default();
        for face in &faces {
            bbox = Aabb::from_aabbs(&bbox, &face.bounding_box());
        }

        Self {
            faces,
            bbox,
            transform: Transform::default(),
        }
    }

    pub fn faces(&self) -> &Vec<Quad> {
        &self.faces
    }
}

impl Hittable for Cuboid {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let ray_obj = Ray::new(
            self.transform.inverse_transform_point(r.origin),
            self.transform.inverse_transform_vector(r.dir),
            r.time,
        );

        let mut hit_record = None;
        let mut closest_so_far = interval.max;

        for face in &self.faces {
            if let Some(mut rec) = face.hit(&ray_obj, &Interval::new(interval.min, closest_so_far))
            {
                closest_so_far = rec.t;

                let normal_world = self.transform.transform_normal(rec.normal);
                rec.point = self.transform.transform_point(rec.point);
                rec.normal = normal_world;
                rec.set_face_normal(r, normal_world);

                hit_record = Some(rec);
            }
        }

        hit_record
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox.transform(&self.transform)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Cuboid {
    fn pdf_value(&self, origin: DVec3, direction: DVec3) -> f64 {
        let weight = 1.0 / self.faces.len() as f64;
        let mut sum = 0.0;
        for face in &self.faces {
            // we'd need to properly inverse-transform origin and dir for accurate pdf
            sum += weight * face.pdf_value(origin, direction);
        }
        sum
    }

    fn random(&self, origin: DVec3) -> DVec3 {
        let int_size = self.faces.len();
        let rand_idx = (crate::utils::random_f64() * int_size as f64).floor() as usize;
        let rand_idx = rand_idx.clamp(0, int_size.saturating_sub(1));
        self.faces[rand_idx].random(origin)
    }
}
