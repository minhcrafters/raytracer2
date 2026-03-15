use std::f64::EPSILON;

use crate::{
    image::Color,
    material::Material,
    ray::{Ray, hit::HitRecord},
    utils::{near_zero, random_unit_vec3},
};

pub struct Metallic {
    pub albedo: Color,
}

impl Metallic {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Metallic {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = ray_in.dir.reflect(hit_record.normal);
        let scattered_ray = Ray::new(hit_record.point, reflected);
        Some((self.albedo, scattered_ray))
    }
}
