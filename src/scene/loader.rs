use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glam::{DQuat, DVec3};

use crate::{
    background::{Background, hdri::Hdri},
    camera::Camera,
    hittable::{
        Hittable, HittableList, cuboid::Cuboid, model::TriMesh, quad::Quad, sphere::Sphere,
        triangle::Triangle,
    },
    image::Color,
    material::{
        Material, dielectric::Dielectric, diffuse_light::DiffuseLight, lambertian::Lambertian,
        metallic::Metallic, specular::Specular,
    },
    ray::transform::Transform,
};

use super::config::{
    BackgroundConfig, ColorArr, MaterialConfig, ObjectConfig, SceneConfig, TransformOpConfig, Vec3,
};

pub struct Scene {
    pub camera: Camera,
    pub world: HittableList,
    pub lights: HittableList,
    pub output: Option<String>,
}

pub fn load_scene(config: SceneConfig) -> Result<Scene, String> {
    _load_scene(config, Path::new("."))
}

pub fn load_scene_from_file(path: &str) -> Result<Scene, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read scene file '{}': {}", path, e))?;

    let config: SceneConfig =
        serde_json::from_str(&text).map_err(|e| format!("Scene JSON parse error: {}", e))?;

    let base_dir = Path::new(path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));

    _load_scene(config, base_dir)
}

pub fn load_scene_from_str(json: &str) -> Result<Scene, String> {
    let config: SceneConfig =
        serde_json::from_str(json).map_err(|e| format!("Scene JSON parse error: {}", e))?;
    _load_scene(config, Path::new("."))
}

fn _load_scene(config: SceneConfig, base_dir: &Path) -> Result<Scene, String> {
    let (image_width, image_height, spp, max_depth, output) = render_params(&config);

    let camera = build_camera(&config, image_width, image_height, spp, max_depth, base_dir);

    let material_map = build_material_map(&config.materials);

    let light_ids: HashSet<&str> = config
        .lights
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect();

    let (world, lights) = build_objects(&config.objects, &material_map, &light_ids, base_dir)?;

    Ok(Scene {
        camera,
        world,
        lights,
        output,
    })
}

fn resolve_path(base_dir: &Path, path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

#[inline]
fn to_dvec3(v: &Vec3) -> DVec3 {
    DVec3::new(v[0], v[1], v[2])
}

#[inline]
fn to_color(c: &ColorArr) -> Color {
    Color::new(c[0], c[1], c[2])
}

fn build_transform(ops: Option<&Vec<TransformOpConfig>>) -> Transform {
    let mut t = Transform::new();
    let Some(ops) = ops else { return t };

    for op in ops {
        match op {
            TransformOpConfig::Scale { value } => {
                t = t.scale(to_dvec3(value));
            }
            TransformOpConfig::Rotate { value } => {
                let quat = DQuat::from_euler(
                    glam::EulerRot::XYZ,
                    value[0].to_radians(),
                    value[1].to_radians(),
                    value[2].to_radians(),
                );
                t = t.rotate(quat);
            }
            TransformOpConfig::Translate { value } => {
                t = t.translate(to_dvec3(value));
            }
        }
    }
    t
}

fn render_params(config: &SceneConfig) -> (usize, usize, usize, usize, Option<String>) {
    match &config.render {
        Some(r) => (
            r.image_width as usize,
            r.image_height as usize,
            r.spp as usize,
            r.max_depth as usize,
            Some(r.output.clone()),
        ),
        None => (800, 450, 100, 50, None),
    }
}

fn build_camera(
    config: &SceneConfig,
    image_width: usize,
    image_height: usize,
    spp: usize,
    max_depth: usize,
    base_dir: &Path,
) -> Camera {
    let mut camera = Camera::new(image_width, image_height, spp, max_depth);
    let cam = &config.camera;

    camera.fov = cam.fov;
    camera.look_from = to_dvec3(&cam.look_from);
    camera.look_at = to_dvec3(&cam.look_at);
    camera.vup = to_dvec3(&cam.vup);
    camera.defocus_angle = cam.defocus_angle;

    camera.background = match &config.background {
        BackgroundConfig::Color { color } => Background::Color(to_color(color)),
        BackgroundConfig::Hdri { path } => {
            let full = resolve_path(base_dir, path);
            Background::Hdri(Hdri::new(full.to_string_lossy().as_ref()))
        }
    };

    camera
}

fn build_material_map(
    materials: &HashMap<String, MaterialConfig>,
) -> HashMap<String, Arc<dyn Material>> {
    materials
        .iter()
        .map(|(name, cfg)| {
            let mat: Arc<dyn Material> = match cfg {
                MaterialConfig::Lambertian { albedo } => {
                    Arc::new(Lambertian::new(to_color(albedo)))
                }
                MaterialConfig::Metallic { albedo, fuzz } => {
                    Arc::new(Metallic::new(to_color(albedo), *fuzz))
                }
                MaterialConfig::Dielectric {
                    color,
                    ior,
                    roughness,
                } => Arc::new(Dielectric::new(to_color(color), *ior, *roughness)),
                MaterialConfig::Specular { albedo, ior, fuzz } => {
                    Arc::new(Specular::new(to_color(albedo), *ior, *fuzz))
                }
                MaterialConfig::DiffuseLight { emit } => {
                    Arc::new(DiffuseLight::new(to_color(emit)))
                }
            };
            (name.clone(), mat)
        })
        .collect()
}

fn resolve_mat(
    name: Option<&str>,
    map: &HashMap<String, Arc<dyn Material>>,
) -> Option<Arc<dyn Material>> {
    match name {
        None => None,
        Some(n) => {
            if let Some(m) = map.get(n) {
                Some(m.clone())
            } else {
                log::warn!(
                    "Scene loader: material '{}' not found; object will have no material",
                    n
                );
                None
            }
        }
    }
}

fn resolve_mat_or_default(
    name: Option<&str>,
    map: &HashMap<String, Arc<dyn Material>>,
) -> Arc<dyn Material> {
    resolve_mat(name, map).unwrap_or_else(|| Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.8))))
}

fn build_objects(
    objects: &[ObjectConfig],
    material_map: &HashMap<String, Arc<dyn Material>>,
    light_ids: &HashSet<&str>,
    base_dir: &Path,
) -> Result<(HittableList, HittableList), String> {
    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    for obj in objects {
        let id = obj.id();
        let is_light = light_ids.contains(id);
        let mat_name = obj.material();
        let transforms = obj.transforms();

        match obj {
            ObjectConfig::Sphere { .. } => {
                let mut sphere = Sphere::new();
                sphere.material = resolve_mat(mat_name, material_map);
                sphere.transform = build_transform(transforms);

                let arc: Arc<dyn Hittable> = Arc::new(sphere);
                if is_light {
                    lights.add(arc.clone());
                }
                world.add(arc);
            }

            ObjectConfig::Quad { q, u, v, .. } => {
                let mat = resolve_mat(mat_name, material_map);

                let mut quad = match (q, u, v) {
                    (Some(q), Some(u), Some(v)) => {
                        Quad::from_points(to_dvec3(q), to_dvec3(u), to_dvec3(v), mat)
                    }
                    _ => {
                        let mut quad = Quad::new();
                        quad.material = mat;
                        quad
                    }
                };
                quad.transform = build_transform(transforms);

                let arc: Arc<dyn Hittable> = Arc::new(quad);
                if is_light {
                    lights.add(arc.clone());
                }
                world.add(arc);
            }

            ObjectConfig::Cuboid { min, max, .. } => {
                let mat = resolve_mat(mat_name, material_map);
                let mut cuboid = Cuboid::new(to_dvec3(min), to_dvec3(max), mat);
                cuboid.transform = build_transform(transforms);

                let arc: Arc<dyn Hittable> = Arc::new(cuboid);
                if is_light {
                    lights.add(arc.clone());
                }
                world.add(arc);
            }

            ObjectConfig::Triangle { a, b, c, .. } => {
                let mat = resolve_mat(mat_name, material_map);
                let q = to_dvec3(a);
                let u = to_dvec3(b) - q;
                let v = to_dvec3(c) - q;

                let mut triangle = Triangle::from_points(q, u, v, mat);
                triangle.transform = build_transform(transforms);

                let arc: Arc<dyn Hittable> = Arc::new(triangle);
                if is_light {
                    lights.add(arc.clone());
                }
                world.add(arc);
            }

            // ----------------------------------------------------------------
            // Trimesh – may produce multiple sub-meshes from a single file.
            //
            // `material` in config controls override behaviour:
            //   • Specified → forced onto every sub-mesh regardless of what
            //     the model file says (useful for scenes like a glass teapot
            //     where you want a specific material applied to the whole mesh).
            //   • Absent    → each sub-mesh keeps the material that
            //     `convert_material` resolved from the model file, with a grey
            //     Lambertian as the ultimate fallback.
            // ----------------------------------------------------------------
            ObjectConfig::Trimesh {
                path,
                recompute_normals,
                ..
            } => {
                // When material is specified it doubles as the fallback passed
                // to load_model AND as the forced per-mesh override below.
                let fallback_mat = resolve_mat_or_default(mat_name, material_map);
                let force_override = mat_name.is_some();

                let smooth = recompute_normals.unwrap_or(false);
                let transform = build_transform(transforms);
                let full_path = resolve_path(base_dir, path);
                let full_path_str = full_path.to_string_lossy();

                match TriMesh::load_model(full_path_str.as_ref(), fallback_mat.clone(), smooth) {
                    Ok(meshes) => {
                        for mut mesh in meshes {
                            mesh.transform = transform;
                            if force_override {
                                // Config `material` specified: apply it to
                                // every sub-mesh, ignoring model-file data.
                                mesh.material = fallback_mat.clone();
                            }
                            // else: keep the per-mesh material that
                            // load_model resolved via convert_material.
                            let arc: Arc<dyn Hittable> = Arc::new(mesh);
                            if is_light {
                                lights.add(arc.clone());
                            }
                            world.add(arc);
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Scene loader: failed to load trimesh '{}' (id='{}'): {}",
                            full_path_str,
                            id,
                            e
                        );
                        // Non-fatal – continue with remaining objects.
                    }
                }
            }
        }
    }

    Ok((world, lights))
}
