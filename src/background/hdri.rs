use glam::DVec3;
use std::path::Path;

use crate::image::Color;

pub struct Hdri {
    pixels: Vec<Color>,
    width: usize,
    height: usize,
}

impl Hdri {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let img = image::open(path)
            .expect("Failed to load HDRI image")
            .to_rgb32f();
        let width = img.width() as usize;
        let height = img.height() as usize;

        let mut pixels = Vec::with_capacity(width * height);
        for pixel in img.pixels() {
            pixels.push(Color::new(
                pixel[0] as f64,
                pixel[1] as f64,
                pixel[2] as f64,
            ));
        }

        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn sample(&self, dir: DVec3) -> Color {
        // Convert direction to spherical coordinates for equirectangular projection
        let dir = dir.normalize();
        let phi = dir.z.atan2(dir.x);
        let theta = dir.y.asin();

        // Map to UV coordinates
        let u = 1.0 - (phi + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
        let v = (theta + std::f64::consts::FRAC_PI_2) / std::f64::consts::PI;

        // Map UV to pixel coordinates
        let i = (u * self.width as f64) as usize;
        let j = ((1.0 - v) * self.height as f64) as usize;

        let i = i.clamp(0, self.width.saturating_sub(1));
        let j = j.clamp(0, self.height.saturating_sub(1));

        self.pixels[j * self.width + i]
    }

    pub fn get_data(&self) -> (&[Color], u32, u32) {
        (&self.pixels, self.width as u32, self.height as u32)
    }
}
