use std::sync::Arc;

use glam::DVec3;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::{Ray, aabb::Aabb, interval::Interval},
};

pub struct Sphere {
    pub center: DVec3,
    pub center_vec: DVec3,
    pub is_moving: bool,
    pub radius: f64,
    pub material: Option<Arc<dyn Material>>,
    pub bbox: Aabb,
}

impl Sphere {
    pub fn stationary(center: DVec3, radius: f64, material: Option<Arc<dyn Material>>) -> Self {
        let rvec = DVec3::new(radius, radius, radius);
        let bbox = Aabb::from_points(center - rvec, center + rvec);
        Self {
            center,
            center_vec: DVec3::ZERO,
            is_moving: false,
            radius,
            material,
            bbox,
        }
    }

    pub fn moving(
        center1: DVec3,
        center2: DVec3,
        radius: f64,
        material: Option<Arc<dyn Material>>,
    ) -> Self {
        let rvec = DVec3::new(radius, radius, radius);
        let box1 = Aabb::from_points(center1 - rvec, center1 + rvec);
        let box2 = Aabb::from_points(center2 - rvec, center2 + rvec);
        let bbox = Aabb::from_aabbs(&box1, &box2);
        Self {
            center: center1,
            center_vec: center2 - center1,
            is_moving: true,
            radius,
            material,
            bbox,
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
            u: 0.0,
            v: 0.0,
        };

        rec.set_face_normal(r, normal);

        Some(rec)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn pdf_value(&self, origin: DVec3, direction: DVec3) -> f64 {
        let ray = Ray::new(origin, direction, 0.0);
        if self
            .hit(&ray, &Interval::new(0.001, f64::INFINITY))
            .is_some()
        {
            let cos_theta_max = (1.0
                - self.radius * self.radius / (self.center - origin).length_squared())
            .max(0.0)
            .sqrt();
            let solid_angle = 2.0 * std::f64::consts::PI * (1.0 - cos_theta_max);
            1.0 / solid_angle
        } else {
            0.0
        }
    }

    fn random(&self, origin: DVec3) -> DVec3 {
        let direction = self.center - origin;
        let distance_squared = direction.length_squared();
        let uvw = crate::utils::OrthonormalBasis::build_from_w(direction);
        let random_to_sphere = crate::utils::random_to_sphere(self.radius, distance_squared);
        uvw.local(random_to_sphere)
    }
}
