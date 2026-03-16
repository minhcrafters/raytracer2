use std::f64::EPSILON;

use crate::{
    image::Color,
    material::Material,
    ray::{Ray, hittable::HitRecord},
    utils::{near_zero, random_unit_vec3},
};

pub struct Metallic {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metallic {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metallic {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let reflected =
            ray_in.dir.reflect(hit_record.normal).normalize() + self.fuzz * random_unit_vec3();
        let scattered_ray = Ray::new(hit_record.point, reflected, ray_in.time);

        if scattered_ray.dir.dot(hit_record.normal) > 0.0 {
            Some((self.albedo, scattered_ray))
        } else {
            None
        }
    }
}
