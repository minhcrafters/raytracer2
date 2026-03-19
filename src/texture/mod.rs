pub mod image;
pub mod solid_color;

use crate::image::Color;
use glam::DVec3;

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: DVec3) -> Color;
    fn alpha(&self, _u: f64, _v: f64, _p: DVec3) -> f64 {
        1.0
    }
    fn as_any(&self) -> &dyn std::any::Any;
}
