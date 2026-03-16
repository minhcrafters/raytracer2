pub mod dielectric;
pub mod diffuse_light;
pub mod lambertian;
pub mod metallic;

use crate::{
    image::Color,
    ray::{Ray, hittable::HitRecord},
};

pub trait Material: Send + Sync {
    /// Scatters an incident ray against the material.
    /// Returns `Some((attenuation, scattered_ray))` if the ray is scattered,
    /// or `None` if the ray is absorbed.
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)>;

    fn emitted(&self, _u: f64, _v: f64, _p: glam::DVec3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}
