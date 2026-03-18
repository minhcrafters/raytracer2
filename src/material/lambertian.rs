use crate::{
    hittable::HitRecord,
    image::Color,
    material::{Material, ScatterRecord},
    pdf::CosinePdf,
    ray::Ray,
    texture::{Texture, solid_color::SolidColor},
};
use std::f64::consts::PI;
use std::sync::Arc;

pub struct Lambertian {
    pub albedo: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(albedo)),
        }
    }

    pub fn with_texture(albedo: Arc<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter<'a>(&self, _ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord<'a>> {
        Some(ScatterRecord {
            attenuation: self
                .albedo
                .value(hit_record.u, hit_record.v, hit_record.point),
            pdf: Some(Box::new(CosinePdf::new(hit_record.normal))),
            skip_pdf: false,
            skip_pdf_ray: Ray::new(hit_record.point, hit_record.normal, 0.0),
        })
    }

    fn scattering_pdf(&self, _ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = hit_record.normal.dot(scattered.dir.normalize());
        if cosine < 0.0 { 0.0 } else { cosine / PI }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Lambertian {
    pub fn get_albedo(&self) -> Arc<dyn Texture> {
        self.albedo.clone()
    }
}
