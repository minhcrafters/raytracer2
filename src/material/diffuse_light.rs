use crate::{hittable::HitRecord, image::Color, ray::Ray};

use super::Material;

pub struct DiffuseLight {
    pub emit: Color,
}

impl DiffuseLight {
    pub fn new(emit: Color) -> Self {
        Self { emit }
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<(Color, Ray)> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _p: glam::DVec3) -> Color {
        self.emit
    }
}
