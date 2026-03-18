pub mod dielectric;
pub mod diffuse_light;
pub mod lambertian;
pub mod metallic;
pub mod specular;

use crate::{hittable::HitRecord, image::Color, ray::Ray};

pub struct ScatterRecord<'a> {
    pub attenuation: Color,
    pub pdf: Option<Box<dyn crate::pdf::Pdf + 'a>>,
    pub skip_pdf: bool,
    pub skip_pdf_ray: Ray,
}

pub trait Material: Send + Sync {
    fn scatter<'a>(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<ScatterRecord<'a>> {
        None
    }

    fn scattering_pdf(&self, _ray_in: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> f64 {
        0.0
    }

    fn emitted(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}
