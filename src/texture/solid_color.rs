use crate::image::Color;
use crate::texture::Texture;
use glam::DVec3;

pub struct SolidColor {
    color_value: Color,
}

impl SolidColor {
    pub fn new(c: Color) -> Self {
        Self { color_value: c }
    }

    pub fn from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self {
            color_value: Color::new(r, g, b),
        }
    }

    pub fn get_color(&self) -> Color {
        self.color_value
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: DVec3) -> Color {
        self.color_value
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
