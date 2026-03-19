use std::path::Path;
use std::sync::Arc;

use asset_importer::{Importer, material::Material as AiMaterial, postprocess::PostProcessSteps};
use glam::{DVec2, DVec3};

use crate::{
    hittable::{HitRecord, Hittable},
    image::Color,
    material::{
        Material, dielectric::Dielectric, diffuse_light::DiffuseLight, lambertian::Lambertian,
        metallic::Metallic, specular::Specular,
    },
    ray::{Ray, aabb::Aabb, interval::Interval, transform::Transform},
    texture::image::ImageTexture,
};

pub struct TriMesh {
    pub vertices: Vec<DVec3>,
    pub normals: Vec<DVec3>,
    pub uvs: Vec<DVec2>,
    pub indices: Vec<u32>,
    pub material: Arc<dyn Material>,
    pub smooth_normals: bool,
    pub transform: Transform,
    pub bvh_root: TriBvhNode,
}

pub struct TriBvhNode {
    pub bbox: Aabb,
    pub left: Option<Box<TriBvhNode>>,
    pub right: Option<Box<TriBvhNode>>,
    pub start_tri: usize,
    pub tri_count: usize,
}

#[derive(Clone, Copy)]
struct Face {
    i0: u32,
    i1: u32,
    i2: u32,
    bbox: Aabb,
    centroid: DVec3,
}

impl TriBvhNode {
    const NUM_SAH_BINS: usize = 12;
    const TRAVERSAL_COST: f64 = 1.0;
    const INTERSECT_COST: f64 = 1.0;
    const MAX_LEAF_TRIS: usize = 4;

    fn build(faces: &mut [Face], start_idx: usize) -> Self {
        let mut bbox = Aabb::default();
        for face in faces.iter() {
            bbox = Aabb::from_aabbs(&bbox, &face.bbox);
        }

        let num_tris = faces.len();
        if num_tris <= Self::MAX_LEAF_TRIS {
            return Self {
                bbox,
                left: None,
                right: None,
                start_tri: start_idx,
                tri_count: num_tris,
            };
        }

        let parent_sa = surface_area(&bbox);
        let leaf_cost = Self::INTERSECT_COST * num_tris as f64;

        let mut best_cost = f64::INFINITY;
        let mut best_axis = 0usize;
        let mut best_split = 0usize;

        for axis in 0..3 {
            let axis_size = bbox.axis(axis).size();
            if axis_size < 1e-10 {
                continue;
            }
            let axis_min = bbox.axis(axis).min;

            let mut bins_count = [0usize; Self::NUM_SAH_BINS];
            let mut bins_bbox = [Aabb::default(); Self::NUM_SAH_BINS];

            for face in faces.iter() {
                let bin = ((face.centroid[axis] - axis_min) / axis_size
                    * Self::NUM_SAH_BINS as f64) as usize;
                let bin = bin.min(Self::NUM_SAH_BINS - 1);
                bins_count[bin] += 1;
                bins_bbox[bin] = Aabb::from_aabbs(&bins_bbox[bin], &face.bbox);
            }

            // left sweep
            let mut left_count = [0usize; Self::NUM_SAH_BINS - 1];
            let mut left_area = [0.0f64; Self::NUM_SAH_BINS - 1];
            let mut running_bbox = Aabb::default();
            let mut running_count = 0usize;
            for i in 0..(Self::NUM_SAH_BINS - 1) {
                running_count += bins_count[i];
                running_bbox = Aabb::from_aabbs(&running_bbox, &bins_bbox[i]);
                left_count[i] = running_count;
                left_area[i] = surface_area(&running_bbox);
            }

            // right sweep
            let mut right_count = [0usize; Self::NUM_SAH_BINS - 1];
            let mut right_area = [0.0f64; Self::NUM_SAH_BINS - 1];
            running_bbox = Aabb::default();
            running_count = 0;
            for i in (0..(Self::NUM_SAH_BINS - 1)).rev() {
                running_count += bins_count[i + 1];
                running_bbox = Aabb::from_aabbs(&running_bbox, &bins_bbox[i + 1]);
                right_count[i] = running_count;
                right_area[i] = surface_area(&running_bbox);
            }

            for i in 0..(Self::NUM_SAH_BINS - 1) {
                if left_count[i] == 0 || right_count[i] == 0 {
                    continue;
                }
                let cost = Self::TRAVERSAL_COST
                    + Self::INTERSECT_COST
                        * (left_count[i] as f64 * left_area[i]
                            + right_count[i] as f64 * right_area[i])
                        / parent_sa;
                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_split = i;
                }
            }
        }

        // fall back to midpoint split when SAH finds no improvement
        if best_cost >= leaf_cost {
            let axis = best_axis;
            faces.sort_by(|a, b| {
                a.centroid[axis]
                    .partial_cmp(&b.centroid[axis])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mid = num_tris / 2;
            let (left_faces, right_faces) = faces.split_at_mut(mid);
            return Self {
                bbox,
                left: Some(Box::new(Self::build(left_faces, start_idx))),
                right: Some(Box::new(Self::build(right_faces, start_idx + mid))),
                start_tri: 0,
                tri_count: 0,
            };
        }

        // partition by the chosen split plane
        let axis_min = bbox.axis(best_axis).min;
        let axis_size = bbox.axis(best_axis).size();
        let split_pos =
            axis_min + (best_split as f64 + 1.0) / Self::NUM_SAH_BINS as f64 * axis_size;

        faces.sort_by(|a, b| {
            a.centroid[best_axis]
                .partial_cmp(&b.centroid[best_axis])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = faces
            .iter()
            .position(|f| f.centroid[best_axis] >= split_pos)
            .unwrap_or(num_tris / 2)
            .clamp(1, num_tris - 1);

        let (left_faces, right_faces) = faces.split_at_mut(mid);

        Self {
            bbox,
            left: Some(Box::new(Self::build(left_faces, start_idx))),
            right: Some(Box::new(Self::build(right_faces, start_idx + mid))),
            start_tri: 0,
            tri_count: 0,
        }
    }

    fn hit(&self, mesh: &TriMesh, ray_obj: &Ray, interval: &Interval) -> Option<HitRecord> {
        if !self.bbox.hit(ray_obj, *interval) {
            return None;
        }

        if self.tri_count > 0 {
            let mut hit_record = None;
            let mut closest_so_far = interval.max;

            for i in 0..self.tri_count {
                let idx = (self.start_tri + i) * 3;
                let i0 = mesh.indices[idx] as usize;
                let i1 = mesh.indices[idx + 1] as usize;
                let i2 = mesh.indices[idx + 2] as usize;

                let p0 = mesh.vertices[i0];
                let p1 = mesh.vertices[i1];
                let p2 = mesh.vertices[i2];

                let edge1 = p1 - p0;
                let edge2 = p2 - p0;
                let h = ray_obj.dir.cross(edge2);
                let a = edge1.dot(h);

                if a > -1e-8 && a < 1e-8 {
                    continue;
                }

                let f = 1.0 / a;
                let s = ray_obj.origin - p0;
                let u = f * s.dot(h);

                if !(0.0..=1.0).contains(&u) {
                    continue;
                }

                let q = s.cross(edge1);
                let v = f * ray_obj.dir.dot(q);

                if v < 0.0 || u + v > 1.0 {
                    continue;
                }

                let t = f * edge2.dot(q);

                if t > interval.min && t < closest_so_far {
                    closest_so_far = t;
                    let intersection_obj = ray_obj.at(t);

                    let w = 1.0 - u - v;
                    let normal_obj = if mesh.smooth_normals && !mesh.normals.is_empty() {
                        let n0 = mesh.normals[i0];
                        let n1 = mesh.normals[i1];
                        let n2 = mesh.normals[i2];
                        (w * n0 + u * n1 + v * n2).normalize()
                    } else {
                        edge1.cross(edge2).normalize()
                    };

                    let (tex_u, tex_v) = if !mesh.uvs.is_empty() {
                        let uv0 = mesh.uvs[i0];
                        let uv1 = mesh.uvs[i1];
                        let uv2 = mesh.uvs[i2];
                        let interp = w * uv0 + u * uv1 + v * uv2;
                        (interp.x, interp.y)
                    } else {
                        (u, v)
                    };

                    hit_record = Some(HitRecord {
                        point: intersection_obj,
                        normal: normal_obj,
                        material: None,
                        t,
                        u: tex_u,
                        v: tex_v,
                        front_face: false,
                    });
                }
            }
            return hit_record;
        }

        let hit_left = self.left.as_ref().unwrap().hit(mesh, ray_obj, interval);

        let right_interval = if let Some(ref rec) = hit_left {
            Interval::new(interval.min, rec.t)
        } else {
            *interval
        };

        let hit_right = self
            .right
            .as_ref()
            .unwrap()
            .hit(mesh, ray_obj, &right_interval);

        hit_right.or(hit_left)
    }
}

fn surface_area(bbox: &Aabb) -> f64 {
    let dx = bbox.x.size().max(0.0);
    let dy = bbox.y.size().max(0.0);
    let dz = bbox.z.size().max(0.0);
    2.0 * (dx * dy + dy * dz + dz * dx)
}

fn convert_material(
    mat: &AiMaterial,
    parent_dir: &Path,
    default_mat: &Arc<dyn Material>,
) -> Arc<dyn Material> {
    let albedo_color = || -> Option<Color> {
        // diffuse (wavefront obj) -> base color (gltf/pbr)
        // check diffuse first because base_color often defaults to white in some importers
        if let Some(c) = mat.diffuse_color() {
            Some(Color::new(c.x as f64, c.y as f64, c.z as f64))
        } else {
            mat.base_color()
                .map(|c| Color::new(c.x as f64, c.y as f64, c.z as f64))
        }
    };

    let opacity = mat.opacity().unwrap_or(1.0);
    let transmission = mat.transmission_factor().unwrap_or(0.0);

    if opacity < 0.5 || transmission > 0.5 {
        let color = albedo_color().unwrap_or(Color::WHITE);
        let ior = mat.refraction_index().unwrap_or(1.5) as f64;
        let roughness = mat.roughness_factor().unwrap_or(0.0).clamp(0.0, 1.0) as f64;
        return Arc::new(Dielectric::new(color, ior, roughness));
    }

    if let Some(emit) = mat.emissive_color() {
        if emit.x > 0.01 || emit.y > 0.01 || emit.z > 0.01 {
            let scale = mat.emissive_intensity().unwrap_or(1.0) as f64;
            let color = Color::new(
                emit.x as f64 * scale,
                emit.y as f64 * scale,
                emit.z as f64 * scale,
            );
            return Arc::new(DiffuseLight::new(color));
        }
    }

    let metallic = mat.metallic_factor().unwrap_or(0.0);
    if metallic >= 0.5 {
        let albedo = albedo_color().unwrap_or(Color::WHITE);
        let fuzz = mat.roughness_factor().unwrap_or(0.5).clamp(0.0, 1.0) as f64;
        return Arc::new(Metallic::new(albedo, fuzz));
    }

    if let Some(tex) = mat
        .albedo_texture(0)
        .or_else(|| mat.base_color_texture(0))
        .or_else(|| mat.texture(asset_importer::material::TextureType::Diffuse, 0))
    {
        // Many material files use backslashes which fail on some platforms or if the model was made on Windows but loaded elsewhere
        let clean_path = tex.path.replace("\\", "/");
        let tex_path = parent_dir.join(&clean_path);
        if let Some(p) = tex_path.to_str() {
            return Arc::new(Lambertian::with_texture(Arc::new(ImageTexture::new(p))));
        }
    }

    let shininess = mat.shininess().unwrap_or(0.0);
    if shininess >= 64.0 {
        // the "albedo" color is used for the diffuse component
        // prefer the albedo (diffuse) color over the specular highlighted color
        let color = albedo_color()
            .or_else(|| {
                mat.specular_color()
                    .map(|c| Color::new(c.x as f64, c.y as f64, c.z as f64))
            })
            .unwrap_or(Color::WHITE);
        let ior = mat.refraction_index().unwrap_or(1.5) as f64;
        return Arc::new(Specular::new(color, ior, shininess as f64));
    }

    if let Some(color) = albedo_color() {
        return Arc::new(Lambertian::new(color));
    }

    default_mat.clone()
}

impl TriMesh {
    pub fn load_model(
        path: &str,
        default_mat: Arc<dyn Material>,
        smooth_normals: bool,
    ) -> Result<Vec<TriMesh>, Box<dyn std::error::Error>> {
        let scene = Importer::new()
            .read_file(path)
            .with_post_process(
                PostProcessSteps::TRIANGULATE
                    | PostProcessSteps::FLIP_UVS
                    | PostProcessSteps::JOIN_IDENTICAL_VERTICES
                    | PostProcessSteps::GEN_SMOOTH_NORMALS
                    | PostProcessSteps::GEN_UV_COORDS,
            )
            .import()?;

        let parent_dir = Path::new(path).parent().unwrap_or(Path::new(""));

        let loaded_materials: Vec<Arc<dyn Material>> = scene
            .materials()
            .map(|mat| convert_material(&mat, parent_dir, &default_mat))
            .collect();

        let mut meshes = Vec::new();

        for mesh in scene.meshes() {
            let vertices: Vec<DVec3> = mesh
                .vertices_iter()
                .map(|v| DVec3::new(v.x as f64, v.y as f64, v.z as f64))
                .collect();

            let normals: Vec<DVec3> = mesh
                .normals_iter()
                .map(|n| DVec3::new(n.x as f64, n.y as f64, n.z as f64))
                .collect();

            let uvs: Vec<DVec2> = if mesh.has_texture_coords(0) {
                mesh.texture_coords_iter2(0)
                    .map(|uv| DVec2::new(uv.x as f64, uv.y as f64))
                    .collect()
            } else {
                Vec::new()
            };

            let mut raw_indices = Vec::new();
            for face in mesh.faces() {
                let face_indices = face.indices();
                if face_indices.len() == 3 {
                    raw_indices.extend_from_slice(face_indices);
                }
            }

            let mut faces = Vec::with_capacity(raw_indices.len() / 3);
            for i in (0..raw_indices.len()).step_by(3) {
                let i0 = raw_indices[i];
                let i1 = raw_indices[i + 1];
                let i2 = raw_indices[i + 2];

                let p0 = vertices[i0 as usize];
                let p1 = vertices[i1 as usize];
                let p2 = vertices[i2 as usize];

                let min_p = p0.min(p1).min(p2);
                let max_p = p0.max(p1).max(p2);
                let mut bbox = Aabb::from_points(min_p, max_p);

                let delta = 0.0001;
                if bbox.x.size() < delta {
                    bbox.x = Interval::new(bbox.x.min - delta / 2.0, bbox.x.max + delta / 2.0);
                }
                if bbox.y.size() < delta {
                    bbox.y = Interval::new(bbox.y.min - delta / 2.0, bbox.y.max + delta / 2.0);
                }
                if bbox.z.size() < delta {
                    bbox.z = Interval::new(bbox.z.min - delta / 2.0, bbox.z.max + delta / 2.0);
                }

                faces.push(Face {
                    i0,
                    i1,
                    i2,
                    bbox,
                    centroid: (p0 + p1 + p2) / 3.0,
                });
            }

            let bvh_root = TriBvhNode::build(&mut faces, 0);

            let indices: Vec<u32> = faces.iter().flat_map(|f| [f.i0, f.i1, f.i2]).collect();

            let material = loaded_materials
                .get(mesh.material_index())
                .cloned()
                .unwrap_or_else(|| default_mat.clone());

            meshes.push(TriMesh {
                vertices,
                normals,
                uvs,
                indices,
                material,
                smooth_normals,
                transform: Transform::default(),
                bvh_root,
            });
        }

        Ok(meshes)
    }
}

impl Hittable for TriMesh {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let ray_obj = Ray::new(
            self.transform.inverse_transform_point(r.origin),
            self.transform.inverse_transform_vector(r.dir),
            r.time,
        );

        if let Some(mut rec) = self.bvh_root.hit(self, &ray_obj, interval) {
            let normal_world = self.transform.transform_normal(rec.normal);
            rec.point = self.transform.transform_point(rec.point);
            rec.normal = normal_world;
            rec.material = Some(self.material.clone());
            rec.set_face_normal(r, normal_world);
            Some(rec)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bvh_root.bbox.transform(&self.transform)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
