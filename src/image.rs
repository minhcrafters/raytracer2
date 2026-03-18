use std::fs::File;
use std::io::Write;
use std::ops::{Add, Div, Mul, Sub};
use std::path::Path;

use glam::DVec3;
use image::{Rgb, RgbImage};

use crate::utils::tonemap;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[allow(dead_code)]
impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.999).clamp(0.0, 255.0) as u8,
            (self.g * 255.999).clamp(0.0, 255.0) as u8,
            (self.b * 255.999).clamp(0.0, 255.0) as u8,
        )
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    pub fn from_vec3(vec3: DVec3) -> Self {
        Self {
            r: vec3.x,
            g: vec3.y,
            b: vec3.z,
        }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Add<f64> for Color {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        Self {
            r: self.r + rhs,
            g: self.g + rhs,
            b: self.b + rhs,
        }
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl Sub<f64> for Color {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self::Output {
        Self {
            r: self.r - rhs,
            g: self.g - rhs,
            b: self.b - rhs,
        }
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Div for Color {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b,
        }
    }
}

impl Div<f64> for Color {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
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
        Color::new(
            self.data[index] as f64 / 255.0,
            self.data[index + 1] as f64 / 255.0,
            self.data[index + 2] as f64 / 255.0,
        )
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: &Color) {
        let index = (y * self.width + x) * 3;

        let r = (tonemap(color.r) * 255.0) as u8;
        let g = (tonemap(color.g) * 255.0) as u8;
        let b = (tonemap(color.b) * 255.0) as u8;

        self.data[index] = r;
        self.data[index + 1] = g;
        self.data[index + 2] = b;
    }

    pub fn set_pixel_raw(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let index = (y * self.width + x) * 3;
        self.data[index] = r;
        self.data[index + 1] = g;
        self.data[index + 2] = b;
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "P6")?;
        writeln!(file, "{} {}", self.width, self.height)?;
        writeln!(file, "255")?;
        file.write_all(&self.data)?;
        Ok(())
    }

    pub fn to_rgb_image(&self) -> RgbImage {
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.get_pixel(x, y);
                img.put_pixel(x as u32, y as u32, Rgb(color.to_tuple().into()));
            }
        }
        img
    }
}
