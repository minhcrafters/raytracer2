use std::f64::EPSILON;

use crate::{
    image::Color,
    material::Material,
    ray::{Ray, hittable::HitRecord},
    utils::{near_zero, random_unit_vec3},
};

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = hit_record.normal + random_unit_vec3();

        if near_zero(scatter_direction) {
            scatter_direction = hit_record.normal;
        }

        let scattered_ray = Ray::new(hit_record.point, scatter_direction, ray_in.time);
        Some((self.albedo, scattered_ray))
    }
}
