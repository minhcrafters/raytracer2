pub mod config;
pub mod loader;

pub use config::{
    BackgroundConfig, CameraConfig, ColorArr, MaterialConfig, ObjectConfig, RenderConfig,
    SceneConfig, TransformOpConfig, Vec3,
};

pub use loader::{Scene, load_scene, load_scene_from_file, load_scene_from_str};
