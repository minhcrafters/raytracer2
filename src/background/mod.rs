pub mod hdri;

use glam::DVec3;

use crate::{background::hdri::Hdri, image::Color};

pub enum Background {
    Color(Color),
    Hdri(Hdri),
}

impl Background {
    pub fn sample(&self, dir: DVec3) -> Color {
        match self {
            Background::Color(c) => *c,
            Background::Hdri(h) => h.sample(dir),
        }
    }
}
