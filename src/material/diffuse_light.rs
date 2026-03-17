use crate::{hittable::HitRecord, image::Color, ray::Ray};

use super::{Material, ScatterRecord};

pub struct DiffuseLight {
    pub emit: Color,
}

impl DiffuseLight {
    pub fn new(emit: Color) -> Self {
        Self { emit }
    }
}

impl Material for DiffuseLight {
    fn scatter<'a>(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<ScatterRecord<'a>> {
        None
    }

    fn emitted(&self, _ray_in: &Ray, hit_record: &HitRecord) -> Color {
        if hit_record.front_face {
            self.emit
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    }
}
