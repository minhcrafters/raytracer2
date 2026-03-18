use crate::image::Color;
use crate::texture::Texture;
use glam::DVec3;
use image::{DynamicImage, GenericImageView};

pub struct ImageTexture {
    image: Option<DynamicImage>,
    width: u32,
    height: u32,
}

impl ImageTexture {
    pub fn new(path: &str) -> Self {
        if let Ok(img) = image::open(path) {
            let width = img.width();
            let height = img.height();
            Self {
                image: Some(img),
                width,
                height,
            }
        } else {
            Self {
                image: None,
                width: 0,
                height: 0,
            }
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: DVec3) -> Color {
        if self.image.is_none() {
            return Color::new(0.0, 1.0, 1.0); // Cyan debugging color
        }

        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // flip V to image coordinates

        let mut i = (u * self.width as f64) as u32;
        let mut j = (v * self.height as f64) as u32;

        if i >= self.width {
            i = self.width - 1;
        }
        if j >= self.height {
            j = self.height - 1;
        }

        let pixel = self.image.as_ref().unwrap().get_pixel(i, j);
        let rgb = pixel.0;

        let color_scale = 1.0 / 255.0;
        Color::new(
            color_scale * rgb[0] as f64,
            color_scale * rgb[1] as f64,
            color_scale * rgb[2] as f64,
        )
    }
}
