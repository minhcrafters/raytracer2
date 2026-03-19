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
        .unwrap();

    if !status.success() {
        eprintln!("glslc shader compilation failed, using existing SPIR-V binary");
    }
}
