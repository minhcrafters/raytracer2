pub mod background;
pub mod camera;
pub mod gpu;
pub mod hittable;
pub mod image;
pub mod material;
pub mod optim;
pub mod pdf;
pub mod ray;
pub mod scene;
pub mod texture;
pub mod utils;

use clap::Parser;

use crate::{
    background::Background,
    camera::Camera,
    gpu::{renderer::GpuRenderer, serialize::SceneSerializer},
    hittable::HittableList,
    image::PPMImage,
    optim::bvh::BvhNode,
    scene::load_scene_from_file,
};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    scene: String,

    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let mut loaded = load_scene_from_file(&cli.scene).expect("Error loading scene");

    let output = cli
        .output
        .or(loaded.output)
        .unwrap_or_else(|| "output".to_string());

    let spp = loaded.camera.spp as u32;
    let bvh = BvhNode::from_list(&loaded.world);

    let image = render_gpu(&mut loaded.camera, &loaded.world, &loaded.lights, &bvh, spp);

    let output_path = format!("{output}");

    // convert to fully using image soon
    image
        .to_rgb_image()
        .save(&output_path)
        .expect("Failed to save image");

    println!("done");
}

fn render_gpu(
    camera: &mut Camera,
    world: &HittableList,
    lights: &HittableList,
    bvh: &BvhNode,
    spp: u32,
) -> PPMImage {
    let mut serializer = SceneSerializer::new();
    let mut scene = serializer.serialize(camera, world, lights, bvh);

    if let Background::Hdri(ref hdri) = camera.background {
        let (pixels, width, height) = hdri.get_data();
        scene.camera.hdri_width = width as u32;
        scene.camera.hdri_height = height as u32;
        scene.hdri_pixels = pixels
            .iter()
            .map(|c| [c.r as f32, c.g as f32, c.b as f32, 0.0])
            .collect();
    }

    let renderer = GpuRenderer::new();
    renderer.render(&scene, spp)
}
