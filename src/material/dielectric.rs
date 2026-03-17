use glam::DVec3;

use crate::{
    image::Color,
    material::{Material, ScatterRecord},
    ray::Ray,
    utils::random_f64,
};

use crate::hittable::HitRecord;

pub struct Dielectric {
    pub ior: f64,
}

impl Dielectric {
    pub fn new(ior: f64) -> Self {
        Self { ior }
    }
}

impl Material for Dielectric {
    fn scatter<'a>(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord<'a>> {
        let ior = if hit_record.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };

        let unit_dir = ray_in.dir.normalize();

        let cos_theta = (-unit_dir).dot(hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ior * sin_theta > 1.0;
        let direction: DVec3;

        if cannot_refract || Self::reflectance(cos_theta, ior) > random_f64() {
            direction = unit_dir.reflect(hit_record.normal);
        } else {
            direction = unit_dir.refract(hit_record.normal, ior);
        }

        let scattered_ray = Ray::new(hit_record.point, direction, ray_in.time);

        Some(ScatterRecord {
            attenuation: Color::WHITE,
            pdf: None,
            skip_pdf: true,
            skip_pdf_ray: scattered_ray,
        })
    }
}

impl Dielectric {
    fn reflectance(cosine: f64, ior: f64) -> f64 {
        let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}
