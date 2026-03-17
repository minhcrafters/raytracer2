use crate::{
    hittable::HitRecord,
    image::Color,
    material::{Material, ScatterRecord},
    pdf::CosinePdf,
    ray::Ray,
};
use std::f64::consts::PI;

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter<'a>(&self, _ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord<'a>> {
        Some(ScatterRecord {
            attenuation: self.albedo,
            pdf: Some(Box::new(CosinePdf::new(hit_record.normal))),
            skip_pdf: false,
            skip_pdf_ray: Ray::new(hit_record.point, hit_record.normal, 0.0),
        })
    }

    fn scattering_pdf(&self, _ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = hit_record.normal.dot(scattered.dir.normalize());
        if cosine < 0.0 { 0.0 } else { cosine / PI }
    }
}
