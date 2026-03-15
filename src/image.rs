use std::fs::File;
use std::io::Write;
use std::ops::{Add, Div, Mul, Sub};

use glam::DVec3;

use crate::ray::interval::Interval;
use crate::utils::linear_to_gamma;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
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

    pub fn from_float(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    pub fn from_vec3(vec3: DVec3) -> Color {
        Color {
            r: vec3.x,
            g: vec3.y,
            b: vec3.z,
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

        let r = (linear_to_gamma(color.r) * 255.0) as u8;
        let g = (linear_to_gamma(color.g) * 255.0) as u8;
        let b = (linear_to_gamma(color.b) * 255.0) as u8;

        self.data[index] = r;
        self.data[index + 1] = g;
        self.data[index + 2] = b;
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
