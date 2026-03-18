use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=shaders/raytracer.comp");

    let status = Command::new("glslc")
        .args([
            "-fshader-stage=compute",
            "shaders/raytracer.comp",
            "-o",
            "shaders/raytracer.spv",
        ])
        .status()
        .expect("glslc shader compilation failed");

    if !status.success() {
        panic!("glslc shader compilation failed");
    }
}
