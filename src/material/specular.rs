use crate::{
    hittable::HitRecord,
    image::Color,
    material::{Material, ScatterRecord},
    pdf::CosinePdf,
    ray::Ray,
    utils::{random_f64, random_unit_vec3},
};
use std::f64::consts::PI;

pub struct Specular {
    pub albedo: Color,
    pub shininess: f64,
    pub ior: f64,
}

impl Specular {
    pub fn new(albedo: Color, ior: f64, shininess: f64) -> Self {
        Self {
            albedo,
            shininess,
            ior,
        }
    }
}

impl Material for Specular {
    fn scatter<'a>(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord<'a>> {
        let unit_dir = ray_in.dir.normalize();
        let cos_theta = (-unit_dir).dot(hit_record.normal).max(0.0).min(1.0);

        let r0 = ((1.0 - self.ior) / (1.0 + self.ior)).powi(2);
        let reflectance = r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5);

        let is_specular = random_f64() < reflectance;

        if is_specular {
            let fuzz = if self.shininess > 0.0 {
                1.0 / self.shininess
            } else {
                1.0
            };

            let reflected =
                ray_in.dir.reflect(hit_record.normal).normalize() + fuzz * random_unit_vec3();
            let scattered_ray = Ray::new(hit_record.point, reflected, ray_in.time);

            if scattered_ray.dir.dot(hit_record.normal) > 0.0 {
                Some(ScatterRecord {
                    attenuation: Color::new(1.0, 1.0, 1.0),
                    pdf: None,
                    skip_pdf: true,
                    skip_pdf_ray: scattered_ray,
                })
            } else {
                None
            }
        } else {
            Some(ScatterRecord {
                attenuation: self.albedo,
                pdf: Some(Box::new(CosinePdf::new(hit_record.normal))),
                skip_pdf: false,
                skip_pdf_ray: Ray::new(hit_record.point, hit_record.normal, 0.0),
            })
        }
    }

    fn scattering_pdf(&self, _ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = hit_record.normal.dot(scattered.dir.normalize());
        if cosine < 0.0 { 0.0 } else { cosine / PI }
    }
}
