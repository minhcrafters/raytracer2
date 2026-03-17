use std::sync::Arc;

use glam::DVec3;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::{Ray, aabb::Aabb, interval::Interval, transform::Transform},
};

pub struct Sphere {
    pub center: DVec3,
    pub center_vec: DVec3,
    pub is_moving: bool,
    pub radius: f64,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
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
            transform: Transform::default(),
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
            transform: Transform::default(),
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
        let ray_obj = Ray::new(
            self.transform.inverse_transform_point(r.origin),
            self.transform.inverse_transform_vector(r.dir),
            r.time,
        );

        let current_center = self.get_center(ray_obj.time);
        let oc = current_center - ray_obj.origin;
        let a = ray_obj.dir.length_squared();
        let h = ray_obj.dir.dot(oc);
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

        let point_obj = ray_obj.at(root);
        let normal_obj = (point_obj - current_center) / self.radius;

        let point_world = self.transform.transform_point(point_obj);
        let normal_world = self.transform.transform_normal(normal_obj);

        let mut rec = HitRecord {
            point: point_world,
            normal: normal_world,
            material: self.material.clone(),
            t: root,
            front_face: false,
            u: 0.0,
            v: 0.0,
        };

        rec.set_face_normal(r, normal_world);

        Some(rec)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox.transform(&self.transform)
    }

    fn pdf_value(&self, origin: DVec3, direction: DVec3) -> f64 {
        let ray = Ray::new(origin, direction, 0.0);
        if self
            .hit(&ray, &Interval::new(0.001, f64::INFINITY))
            .is_some()
        {
            let center_world = self.transform.transform_point(self.center);
            let radius_world = self
                .transform
                .transform_vector(DVec3::new(self.radius, 0.0, 0.0))
                .length();

            let cos_theta_max = (1.0
                - radius_world * radius_world / (center_world - origin).length_squared())
            .max(0.0)
            .sqrt();
            let solid_angle = 2.0 * std::f64::consts::PI * (1.0 - cos_theta_max);
            1.0 / solid_angle
        } else {
            0.0
        }
    }

    fn random(&self, origin: DVec3) -> DVec3 {
        let center_world = self.transform.transform_point(self.center);
        let radius_world = self
            .transform
            .transform_vector(DVec3::new(self.radius, 0.0, 0.0))
            .length();

        let direction = center_world - origin;
        let distance_squared = direction.length_squared();
        let uvw = crate::utils::OrthonormalBasis::build_from_w(direction);
        let random_to_sphere = crate::utils::random_to_sphere(radius_world, distance_squared);
        uvw.local(random_to_sphere)
    }
}
