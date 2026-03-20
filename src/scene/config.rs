use serde::Deserialize;
use std::collections::HashMap;

pub type Vec3 = [f64; 3];

pub type ColorArr = [f64; 3];

#[derive(Debug, Deserialize)]
pub struct SceneConfig {
    pub render: Option<RenderConfig>,
    pub camera: CameraConfig,
    pub background: BackgroundConfig,
    pub materials: HashMap<String, MaterialConfig>,
    pub objects: Vec<ObjectConfig>,
    pub lights: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct RenderConfig {
    pub image_width: u32,
    pub image_height: u32,
    pub spp: u32,
    pub max_depth: u32,
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub fov: f64,
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub vup: Vec3,
    pub defocus_angle: f64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackgroundConfig {
    Color { color: ColorArr },
    Hdri { path: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MaterialConfig {
    Lambertian {
        albedo: ColorArr,
    },
    Metallic {
        albedo: ColorArr,
        fuzz: f64,
    },
    Dielectric {
        color: ColorArr,
        ior: f64,
        roughness: f64,
    },
    Specular {
        albedo: [f64; 3],
        ior: f64,
        fuzz: f64,
    },
    DiffuseLight {
        emit: ColorArr,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransformOpConfig {
    Scale { value: Vec3 },
    Rotate { value: Vec3 },
    Translate { value: Vec3 },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObjectConfig {
    Sphere {
        id: String,
        material: Option<String>,
        transforms: Option<Vec<TransformOpConfig>>,
    },
    Quad {
        id: String,
        material: Option<String>,
        transforms: Option<Vec<TransformOpConfig>>,
        q: Option<Vec3>,
        u: Option<Vec3>,
        v: Option<Vec3>,
    },
    Cuboid {
        id: String,
        material: Option<String>,
        transforms: Option<Vec<TransformOpConfig>>,
        min: Vec3,
        max: Vec3,
    },
    Triangle {
        id: String,
        material: Option<String>,
        transforms: Option<Vec<TransformOpConfig>>,
        a: Vec3,
        b: Vec3,
        c: Vec3,
    },
    Trimesh {
        id: String,
        material: Option<String>,
        transforms: Option<Vec<TransformOpConfig>>,
        path: String,
        recompute_normals: Option<bool>,
    },
}

impl ObjectConfig {
    pub fn id(&self) -> &str {
        match self {
            ObjectConfig::Sphere { id, .. } => id,
            ObjectConfig::Quad { id, .. } => id,
            ObjectConfig::Cuboid { id, .. } => id,
            ObjectConfig::Triangle { id, .. } => id,
            ObjectConfig::Trimesh { id, .. } => id,
        }
    }

    pub fn material(&self) -> Option<&str> {
        let opt = match self {
            ObjectConfig::Sphere { material, .. } => material,
            ObjectConfig::Quad { material, .. } => material,
            ObjectConfig::Cuboid { material, .. } => material,
            ObjectConfig::Triangle { material, .. } => material,
            ObjectConfig::Trimesh { material, .. } => material,
        };
        opt.as_deref()
    }

    pub fn transforms(&self) -> Option<&Vec<TransformOpConfig>> {
        let opt = match self {
            ObjectConfig::Sphere { transforms, .. } => transforms,
            ObjectConfig::Quad { transforms, .. } => transforms,
            ObjectConfig::Cuboid { transforms, .. } => transforms,
            ObjectConfig::Triangle { transforms, .. } => transforms,
            ObjectConfig::Trimesh { transforms, .. } => transforms,
        };
        opt.as_ref()
    }
}
