use std::fs::File;
use std::io::Write;

use glam::DVec3;

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    pub fn from_vec3(vec3: DVec3) -> Color {
        Color {
            r: (vec3.x * 255.999) as u8,
            g: (vec3.y * 255.999) as u8,
            b: (vec3.z * 255.999) as u8,
        }
    }
}

pub struct PPMImage {
    pub width: usize,
    pub height: usize,
    data: Vec<u8>,
}

impl PPMImage {
    pub fn new(width: usize, height: usize) -> Self {
        let data = vec![0; width * height * 3]; // 3 bytes per pixel (RGB)
        Self {
            width,
            height,
            data,
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        let index = (y * self.width + x) * 3;
        Color::new(self.data[index], self.data[index + 1], self.data[index + 2])
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: &Color) {
        let index = (y * self.width + x) * 3;
        self.data[index] = color.r;
        self.data[index + 1] = color.g;
        self.data[index + 2] = color.b;
    }

    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        writeln!(file, "P6")?;
        writeln!(file, "{} {}", self.width, self.height)?;
        writeln!(file, "255")?;
        file.write_all(&self.data)?;
        Ok(())
    }
}
