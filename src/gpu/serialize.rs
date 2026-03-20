use std::sync::Arc;

use bytemuck::Zeroable;
use glam::{DMat4, DVec3};

use crate::{
    background::Background,
    camera::Camera,
    gpu::buffers::*,
    gpu::renderer::SceneBuffers,
    hittable::{Hittable, HittableList},
    optim::bvh::BvhNode,
};

pub struct SceneSerializer {
    materials: Vec<GpuMaterial>,
    transforms: Vec<GpuTransform>,
    spheres: Vec<GpuSphere>,
    quads: Vec<GpuQuad>,
    primitives: Vec<GpuPrimitive>,
    triangles: Vec<GpuTriangle>,
    tex_pixels: Vec<[f32; 4]>,
    light_prim_indices: Vec<u32>,
    material_map: std::collections::HashMap<usize, u32>,
}

impl SceneSerializer {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
            transforms: Vec::new(),
            spheres: Vec::new(),
            quads: Vec::new(),
            primitives: Vec::new(),
            triangles: Vec::new(),
            tex_pixels: Vec::new(),
            light_prim_indices: Vec::new(),
            material_map: std::collections::HashMap::new(),
        }
    }

    pub fn serialize(
        &mut self,
        camera: &mut Camera,
        world: &HittableList,
        lights: &HittableList,
        bvh: &BvhNode,
    ) -> SceneBuffers {
        camera.init();

        self.serialize_hittable_list(world);

        self.find_lights(world, lights);

        let bvh_nodes = self.flatten_scene_bvh(bvh, world);

        let gpu_camera = self.build_camera(camera);

        // Ensure we have at least one of each to avoid empty buffer issues
        if self.materials.is_empty() {
            self.materials.push(GpuMaterial::zeroed());
        }
        if self.transforms.is_empty() {
            self.transforms.push(GpuTransform::default());
        }
        if self.primitives.is_empty() {
            self.primitives.push(GpuPrimitive::zeroed());
        }

        SceneBuffers {
            camera: gpu_camera,
            materials: self.materials.clone(),
            transforms: self.transforms.clone(),
            spheres: self.spheres.clone(),
            quads: self.quads.clone(),
            primitives: self.primitives.clone(),
            bvh_nodes,
            light_prim_indices: self.light_prim_indices.clone(),
            triangles: self.triangles.clone(),
            hdri_pixels: Vec::new(),
            tex_pixels: self.tex_pixels.clone(),
        }
    }

    fn serialize_hittable_list(&mut self, world: &HittableList) {
        for obj in &world.objects {
            self.serialize_object(obj.as_ref());
        }
    }

    fn serialize_object(&mut self, obj: &dyn Hittable) {
        // Try to downcast to known types
        if let Some(sphere) = obj
            .as_any()
            .downcast_ref::<crate::hittable::sphere::Sphere>()
        {
            self.add_sphere(sphere);
        } else if let Some(quad) = obj.as_any().downcast_ref::<crate::hittable::quad::Quad>() {
            self.add_quad(quad);
        } else if let Some(cuboid) = obj
            .as_any()
            .downcast_ref::<crate::hittable::cuboid::Cuboid>()
        {
            self.add_cuboid(cuboid);
        } else if let Some(trimesh) = obj
            .as_any()
            .downcast_ref::<crate::hittable::model::TriMesh>()
        {
            self.add_trimesh(trimesh);
        }
        // BvhNode is handled separately via flatten_scene_bvh
    }

    fn add_sphere(&mut self, sphere: &crate::hittable::sphere::Sphere) {
        let mat_idx = self.add_material(&sphere.material);
        let tf_idx = self.add_transform(&sphere.transform);
        let sphere_idx = self.spheres.len() as u32;

        self.spheres.push(GpuSphere {
            center: dvec3_to_f32(sphere.center),
            radius: sphere.radius as f32,
        });

        self.primitives.push(GpuPrimitive {
            prim_type: PRIM_SPHERE,
            data_index: sphere_idx,
            material_index: mat_idx,
            transform_index: tf_idx,
        });
    }

    fn add_quad(&mut self, quad: &crate::hittable::quad::Quad) {
        let mat_idx = self.add_material(&quad.material);
        let tf_idx = self.add_transform(&quad.transform);
        let quad_idx = self.quads.len() as u32;

        let n = quad.u.cross(quad.v);
        let normal = n.normalize();
        let d = normal.dot(quad.q);
        let w = n / n.length_squared();

        self.quads.push(GpuQuad {
            q: dvec3_to_f32(quad.q),
            d: d as f32,
            u: dvec3_to_f32(quad.u),
            _pad0: 0.0,
            v: dvec3_to_f32(quad.v),
            _pad1: 0.0,
            normal: dvec3_to_f32(normal),
            _pad2: 0.0,
            w: dvec3_to_f32(w),
            _pad3: 0.0,
        });

        self.primitives.push(GpuPrimitive {
            prim_type: PRIM_QUAD,
            data_index: quad_idx,
            material_index: mat_idx,
            transform_index: tf_idx,
        });
    }

    fn add_cuboid(&mut self, cuboid: &crate::hittable::cuboid::Cuboid) {
        // A cuboid is 6 quads sharing the cuboid's transform
        for face in cuboid.faces() {
            let mat_idx = self.add_material(&face.material);
            let tf_idx = self.add_transform(&cuboid.transform);
            let quad_idx = self.quads.len() as u32;

            let n = face.u.cross(face.v);
            let normal = n.normalize();
            let d = normal.dot(face.q);
            let w = n / n.length_squared();

            self.quads.push(GpuQuad {
                q: dvec3_to_f32(face.q),
                d: d as f32,
                u: dvec3_to_f32(face.u),
                _pad0: 0.0,
                v: dvec3_to_f32(face.v),
                _pad1: 0.0,
                normal: dvec3_to_f32(normal),
                _pad2: 0.0,
                w: dvec3_to_f32(w),
                _pad3: 0.0,
            });

            self.primitives.push(GpuPrimitive {
                prim_type: PRIM_QUAD,
                data_index: quad_idx,
                material_index: mat_idx,
                transform_index: tf_idx,
            });
        }
    }

    fn add_trimesh(&mut self, mesh: &crate::hittable::model::TriMesh) {
        let mat_idx = self.add_material_arc(&mesh.material);
        let tf_idx = self.add_transform(&mesh.transform);

        let num_triangles = mesh.indices.len() / 3;
        let start_triangle = self.triangles.len() as u32;

        for i in 0..num_triangles {
            let i0 = mesh.indices[i * 3] as usize;
            let i1 = mesh.indices[i * 3 + 1] as usize;
            let i2 = mesh.indices[i * 3 + 2] as usize;

            let p0 = mesh.vertices[i0];
            let p1 = mesh.vertices[i1];
            let p2 = mesh.vertices[i2];

            let (n0, n1, n2, has_normals) = if !mesh.normals.is_empty() {
                (mesh.normals[i0], mesh.normals[i1], mesh.normals[i2], 1)
            } else {
                (glam::DVec3::ZERO, glam::DVec3::ZERO, glam::DVec3::ZERO, 0)
            };

            let (uv0, uv1, uv2, has_uvs) = if !mesh.uvs.is_empty() {
                (mesh.uvs[i0], mesh.uvs[i1], mesh.uvs[i2], 1)
            } else {
                (glam::DVec2::ZERO, glam::DVec2::ZERO, glam::DVec2::ZERO, 0)
            };

            self.triangles.push(GpuTriangle {
                v0: dvec3_to_f32(p0),
                _pad0: 0.0,
                v1: dvec3_to_f32(p1),
                _pad1: 0.0,
                v2: dvec3_to_f32(p2),
                _pad2: 0.0,
                n0: dvec3_to_f32(n0),
                _pad3: 0.0,
                n1: dvec3_to_f32(n1),
                _pad4: 0.0,
                n2: dvec3_to_f32(n2),
                _pad5: 0.0,
                uv0: [uv0.x as f32, uv0.y as f32],
                uv1: [uv1.x as f32, uv1.y as f32],
                uv2: [uv2.x as f32, uv2.y as f32],
                has_normals,
                has_uvs,
                _pad6: [0; 4],
            });

            self.primitives.push(GpuPrimitive {
                prim_type: PRIM_TRIANGLE,
                data_index: start_triangle + i as u32,
                material_index: mat_idx,
                transform_index: tf_idx,
            });
        }
    }

    fn flatten_mesh_bvh(
        &self,
        node: &crate::hittable::model::TriBvhNode,
        tf: &crate::ray::transform::Transform,
        start_prim: u32,
        nodes: &mut Vec<GpuBvhNode>,
    ) -> u32 {
        let current_idx = nodes.len() as u32;
        let (bbox_min, bbox_max) = transform_aabb_to_gpu(&node.bbox, tf);

        if node.tri_count > 0 {
            nodes.push(GpuBvhNode {
                bbox_min,
                left_or_prim: start_prim + node.start_tri as u32,
                bbox_max,
                right_or_count: node.tri_count as u32,
                is_leaf: 1,
                _pad: [0; 3],
            });
            current_idx
        } else {
            nodes.push(GpuBvhNode::zeroed());

            let left_idx = if let Some(ref left) = node.left {
                self.flatten_mesh_bvh(left, tf, start_prim, nodes)
            } else {
                0
            };

            let right_idx = if let Some(ref right) = node.right {
                self.flatten_mesh_bvh(right, tf, start_prim, nodes)
            } else {
                0
            };

            nodes[current_idx as usize] = GpuBvhNode {
                bbox_min,
                left_or_prim: left_idx,
                bbox_max,
                right_or_count: right_idx,
                is_leaf: 0,
                _pad: [0; 3],
            };

            current_idx
        }
    }

    fn flatten_scene_bvh(&self, bvh: &BvhNode, world: &HittableList) -> Vec<GpuBvhNode> {
        let mut ptr_to_prim: std::collections::HashMap<usize, Vec<u32>> =
            std::collections::HashMap::new();
        let mut ptr_to_mesh: std::collections::HashMap<
            usize,
            (&crate::hittable::model::TriMesh, u32),
        > = std::collections::HashMap::new();

        let mut prim_idx = 0u32;
        for obj in &world.objects {
            let ptr = Arc::as_ptr(obj) as *const () as usize;
            let start = prim_idx;

            if obj.as_any().is::<crate::hittable::cuboid::Cuboid>() {
                prim_idx += 6;
            } else if let Some(mesh) = obj
                .as_any()
                .downcast_ref::<crate::hittable::model::TriMesh>()
            {
                prim_idx += (mesh.indices.len() / 3) as u32;
                ptr_to_mesh.insert(ptr, (mesh, start));
            } else {
                prim_idx += 1;
            }

            let indices: Vec<u32> = (start..prim_idx).collect();
            ptr_to_prim.insert(ptr, indices);
        }

        let mut nodes = Vec::new();
        let _ = self.flatten_bvh_node_recursive(bvh, &ptr_to_prim, &ptr_to_mesh, &mut nodes);

        if nodes.is_empty() {
            nodes.push(GpuBvhNode {
                bbox_min: [0.0; 3],
                left_or_prim: 0,
                bbox_max: [0.0; 3],
                right_or_count: 0,
                is_leaf: 1,
                _pad: [0; 3],
            });
        }

        nodes
    }

    fn flatten_bvh_node_recursive(
        &self,
        bvh: &BvhNode,
        ptr_to_prim: &std::collections::HashMap<usize, Vec<u32>>,
        ptr_to_mesh: &std::collections::HashMap<usize, (&crate::hittable::model::TriMesh, u32)>,
        nodes: &mut Vec<GpuBvhNode>,
    ) -> u32 {
        let current_idx = nodes.len() as u32;

        let left_ptr = Arc::as_ptr(&bvh.left) as *const () as usize;
        let right_ptr = Arc::as_ptr(&bvh.right) as *const () as usize;

        let left_is_bvh = bvh.left.as_any().is::<BvhNode>();
        let right_is_bvh = bvh.right.as_any().is::<BvhNode>();

        if !left_is_bvh && !right_is_bvh {
            if left_ptr == right_ptr {
                if let Some(&(mesh, start)) = ptr_to_mesh.get(&left_ptr) {
                    return self.flatten_mesh_bvh(&mesh.bvh_root, &mesh.transform, start, nodes);
                } else if let Some(prim_indices) = ptr_to_prim.get(&left_ptr) {
                    nodes.push(GpuBvhNode {
                        bbox_min: aabb_min(&bvh.bbox),
                        left_or_prim: prim_indices[0],
                        bbox_max: aabb_max(&bvh.bbox),
                        right_or_count: prim_indices.len() as u32,
                        is_leaf: 1,
                        _pad: [0; 3],
                    });
                    return current_idx;
                }
            } else {
                nodes.push(GpuBvhNode::zeroed());

                let left_child_idx = if let Some(&(mesh, start)) = ptr_to_mesh.get(&left_ptr) {
                    self.flatten_mesh_bvh(&mesh.bvh_root, &mesh.transform, start, nodes)
                } else if let Some(prim_indices) = ptr_to_prim.get(&left_ptr) {
                    let idx = nodes.len() as u32;
                    nodes.push(GpuBvhNode {
                        bbox_min: aabb_min(&bvh.left.bounding_box()),
                        left_or_prim: prim_indices[0],
                        bbox_max: aabb_max(&bvh.left.bounding_box()),
                        right_or_count: prim_indices.len() as u32,
                        is_leaf: 1,
                        _pad: [0; 3],
                    });
                    idx
                } else {
                    0
                };

                let right_child_idx = if let Some(&(mesh, start)) = ptr_to_mesh.get(&right_ptr) {
                    self.flatten_mesh_bvh(&mesh.bvh_root, &mesh.transform, start, nodes)
                } else if let Some(prim_indices) = ptr_to_prim.get(&right_ptr) {
                    let idx = nodes.len() as u32;
                    nodes.push(GpuBvhNode {
                        bbox_min: aabb_min(&bvh.right.bounding_box()),
                        left_or_prim: prim_indices[0],
                        bbox_max: aabb_max(&bvh.right.bounding_box()),
                        right_or_count: prim_indices.len() as u32,
                        is_leaf: 1,
                        _pad: [0; 3],
                    });
                    idx
                } else {
                    0
                };

                nodes[current_idx as usize] = GpuBvhNode {
                    bbox_min: aabb_min(&bvh.bbox),
                    left_or_prim: left_child_idx,
                    bbox_max: aabb_max(&bvh.bbox),
                    right_or_count: right_child_idx,
                    is_leaf: 0,
                    _pad: [0; 3],
                };
                return current_idx;
            }
        }

        nodes.push(GpuBvhNode::zeroed());

        let left_child_idx = if left_is_bvh {
            let left_bvh = bvh.left.as_any().downcast_ref::<BvhNode>().unwrap();
            self.flatten_bvh_node_recursive(left_bvh, ptr_to_prim, ptr_to_mesh, nodes)
        } else {
            if let Some(&(mesh, start)) = ptr_to_mesh.get(&left_ptr) {
                self.flatten_mesh_bvh(&mesh.bvh_root, &mesh.transform, start, nodes)
            } else if let Some(prim_indices) = ptr_to_prim.get(&left_ptr) {
                let idx = nodes.len() as u32;
                nodes.push(GpuBvhNode {
                    bbox_min: aabb_min(&bvh.left.bounding_box()),
                    left_or_prim: prim_indices[0],
                    bbox_max: aabb_max(&bvh.left.bounding_box()),
                    right_or_count: prim_indices.len() as u32,
                    is_leaf: 1,
                    _pad: [0; 3],
                });
                idx
            } else {
                0
            }
        };

        let right_child_idx = if right_is_bvh {
            let right_bvh = bvh.right.as_any().downcast_ref::<BvhNode>().unwrap();
            self.flatten_bvh_node_recursive(right_bvh, ptr_to_prim, ptr_to_mesh, nodes)
        } else {
            if let Some(&(mesh, start)) = ptr_to_mesh.get(&right_ptr) {
                self.flatten_mesh_bvh(&mesh.bvh_root, &mesh.transform, start, nodes)
            } else if let Some(prim_indices) = ptr_to_prim.get(&right_ptr) {
                let idx = nodes.len() as u32;
                nodes.push(GpuBvhNode {
                    bbox_min: aabb_min(&bvh.right.bounding_box()),
                    left_or_prim: prim_indices[0],
                    bbox_max: aabb_max(&bvh.right.bounding_box()),
                    right_or_count: prim_indices.len() as u32,
                    is_leaf: 1,
                    _pad: [0; 3],
                });
                idx
            } else {
                0
            }
        };

        nodes[current_idx as usize] = GpuBvhNode {
            bbox_min: aabb_min(&bvh.bbox),
            left_or_prim: left_child_idx,
            bbox_max: aabb_max(&bvh.bbox),
            right_or_count: right_child_idx,
            is_leaf: 0,
            _pad: [0; 3],
        };

        current_idx
    }

    fn find_lights(&mut self, world: &HittableList, lights: &HittableList) {
        // Find which primitive indices correspond to light objects
        for light_obj in &lights.objects {
            let light_ptr = Arc::as_ptr(light_obj) as *const () as usize;

            // Search through world objects to find matching pointer
            let mut prim_idx = 0u32;
            for world_obj in &world.objects {
                let world_ptr = Arc::as_ptr(world_obj) as *const () as usize;
                let num_prims = if world_obj.as_any().is::<crate::hittable::cuboid::Cuboid>() {
                    6u32
                } else {
                    1u32
                };

                if world_ptr == light_ptr {
                    for i in 0..num_prims {
                        self.light_prim_indices.push(prim_idx + i);
                    }
                    break;
                }
                prim_idx += num_prims;
            }
        }
    }

    fn add_material(&mut self, mat: &Option<Arc<dyn crate::material::Material>>) -> u32 {
        if let Some(mat) = mat {
            self.add_material_arc(mat)
        } else {
            // Default material (lambertian white)
            let idx = self.materials.len() as u32;
            self.materials.push(GpuMaterial {
                mat_type: MAT_LAMBERTIAN,
                _pad0: [0; 3],
                albedo: [0.8, 0.8, 0.8],
                ior: 1.0,
                emit: [0.0; 3],
                fuzz: 0.0,
                _pad1: 0,
                tex_height: 0,
                tex_width: 0,
                tex_offset: 0,
            });
            idx
        }
    }

    fn add_material_arc(&mut self, mat: &Arc<dyn crate::material::Material>) -> u32 {
        let ptr = Arc::as_ptr(mat) as *const () as usize;
        if let Some(&idx) = self.material_map.get(&ptr) {
            return idx;
        }

        let idx = self.materials.len() as u32;
        let gpu_mat = self.convert_material(mat.as_ref());
        self.materials.push(gpu_mat);
        self.material_map.insert(ptr, idx);
        idx
    }

    fn convert_material(&mut self, mat: &dyn crate::material::Material) -> GpuMaterial {
        use crate::material::{
            dielectric::Dielectric, diffuse_light::DiffuseLight, lambertian::Lambertian,
            metallic::Metallic, specular::Specular,
        };

        let any = mat.as_any();

        if let Some(lam) = any.downcast_ref::<Lambertian>() {
            // Check if it has a texture
            let tex_any = lam.albedo.as_any();
            if let Some(img_tex) = tex_any.downcast_ref::<crate::texture::image::ImageTexture>() {
                if let Some((pixels, width, height)) = img_tex.get_pixel_data() {
                    let tex_offset = self.tex_pixels.len() as u32;
                    for p in &pixels {
                        self.tex_pixels.push(*p);
                    }
                    return GpuMaterial {
                        mat_type: MAT_LAMBERTIAN,
                        _pad0: [0; 3],
                        albedo: [1.0, 1.0, 1.0],
                        ior: 1.0,
                        emit: [0.0; 3],
                        fuzz: 0.0,
                        _pad1: 0,
                        tex_height: height,
                        tex_width: width,
                        tex_offset,
                    };
                }
            }

            // Solid color
            let color = lam.albedo.value(0.0, 0.0, glam::DVec3::ZERO);
            GpuMaterial {
                mat_type: MAT_LAMBERTIAN,
                _pad0: [0; 3],
                albedo: [color.r as f32, color.g as f32, color.b as f32],
                ior: 1.0,
                emit: [0.0; 3],
                fuzz: 0.0,
                _pad1: 0,
                tex_height: 0,
                tex_width: 0,
                tex_offset: 0,
            }
        } else if let Some(met) = any.downcast_ref::<Metallic>() {
            GpuMaterial {
                mat_type: MAT_METALLIC,
                _pad0: [0; 3],
                albedo: [
                    met.albedo.r as f32,
                    met.albedo.g as f32,
                    met.albedo.b as f32,
                ],
                ior: 1.0,
                emit: [0.0; 3],
                fuzz: met.fuzz as f32,
                _pad1: 0,
                tex_height: 0,
                tex_width: 0,
                tex_offset: 0,
            }
        } else if let Some(die) = any.downcast_ref::<Dielectric>() {
            GpuMaterial {
                mat_type: MAT_DIELECTRIC,
                _pad0: [0; 3],
                albedo: [
                    die.albedo.r as f32,
                    die.albedo.g as f32,
                    die.albedo.b as f32,
                ],
                ior: die.ior as f32,
                emit: [0.0; 3],
                fuzz: die.fuzz as f32,
                _pad1: 0,
                tex_height: 0,
                tex_width: 0,
                tex_offset: 0,
            }
        } else if let Some(light) = any.downcast_ref::<DiffuseLight>() {
            GpuMaterial {
                mat_type: MAT_DIFFUSE_LIGHT,
                _pad0: [0; 3],
                albedo: [0.0; 3],
                ior: 1.0,
                emit: [
                    light.emit.r as f32,
                    light.emit.g as f32,
                    light.emit.b as f32,
                ],
                fuzz: 0.0,
                _pad1: 0,
                tex_height: 0,
                tex_width: 0,
                tex_offset: 0,
            }
        } else if let Some(spec) = any.downcast_ref::<Specular>() {
            GpuMaterial {
                mat_type: MAT_SPECULAR,
                _pad0: [0; 3],
                albedo: [
                    spec.albedo.r as f32,
                    spec.albedo.g as f32,
                    spec.albedo.b as f32,
                ],
                ior: spec.ior as f32,
                emit: [0.0; 3],
                fuzz: spec.fuzz as f32,
                _pad1: 0,
                tex_height: 0,
                tex_width: 0,
                tex_offset: 0,
            }
        } else {
            // Fallback
            GpuMaterial {
                mat_type: MAT_LAMBERTIAN,
                _pad0: [0; 3],
                albedo: [0.8, 0.8, 0.8],
                ior: 1.0,
                emit: [0.0; 3],
                fuzz: 0.0,
                _pad1: 0,
                tex_height: 0,
                tex_width: 0,
                tex_offset: 0,
            }
        }
    }

    fn add_transform(&mut self, tf: &crate::ray::transform::Transform) -> u32 {
        let idx = self.transforms.len() as u32;
        self.transforms.push(GpuTransform {
            m: dmat4_to_f32(tf.m),
            inv: dmat4_to_f32(tf.inv),
        });
        idx
    }

    fn build_camera(&self, camera: &Camera) -> GpuCamera {
        GpuCamera {
            inv_view: dmat4_to_f32(camera.inv_view_mat),
            inv_proj: dmat4_to_f32(camera.inv_proj_mat),
            center: dvec3_to_f32(camera.look_from),
            image_width: camera.image_width() as u32,
            defocus_disk_u: dvec3_to_f32(camera.defocus_disk_u()),
            image_height: camera.image_height() as u32,
            defocus_disk_v: dvec3_to_f32(camera.defocus_disk_v()),
            max_depth: camera.max_depth as u32,
            defocus_angle: camera.defocus_angle as f32,
            current_sample: 0,
            bg_type: match &camera.background {
                Background::Color(_) => BG_COLOR,
                Background::Hdri(_) => BG_HDRI,
            },
            bg_color: match &camera.background {
                Background::Color(c) => [c.r as f32, c.g as f32, c.b as f32],
                _ => [0.0; 3],
            },
            hdri_width: 0, // filled later
            hdri_height: 0,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
        }
    }
}

// ─── Helpers ───

fn dvec3_to_f32(v: DVec3) -> [f32; 3] {
    [v.x as f32, v.y as f32, v.z as f32]
}

fn dmat4_to_f32(m: DMat4) -> [[f32; 4]; 4] {
    let cols = m.to_cols_array_2d();
    [
        [
            cols[0][0] as f32,
            cols[0][1] as f32,
            cols[0][2] as f32,
            cols[0][3] as f32,
        ],
        [
            cols[1][0] as f32,
            cols[1][1] as f32,
            cols[1][2] as f32,
            cols[1][3] as f32,
        ],
        [
            cols[2][0] as f32,
            cols[2][1] as f32,
            cols[2][2] as f32,
            cols[2][3] as f32,
        ],
        [
            cols[3][0] as f32,
            cols[3][1] as f32,
            cols[3][2] as f32,
            cols[3][3] as f32,
        ],
    ]
}

fn transform_aabb_to_gpu(
    aabb: &crate::ray::aabb::Aabb,
    tf: &crate::ray::transform::Transform,
) -> ([f32; 3], [f32; 3]) {
    let mut min = glam::DVec3::splat(f64::INFINITY);
    let mut max = glam::DVec3::splat(f64::NEG_INFINITY);

    let x0 = aabb.x.min;
    let x1 = aabb.x.max;
    let y0 = aabb.y.min;
    let y1 = aabb.y.max;
    let z0 = aabb.z.min;
    let z1 = aabb.z.max;

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let p = glam::DVec3::new(
                    if i == 0 { x0 } else { x1 },
                    if j == 0 { y0 } else { y1 },
                    if k == 0 { z0 } else { z1 },
                );
                let p_tf = tf.m.transform_point3(p);
                min = min.min(p_tf);
                max = max.max(p_tf);
            }
        }
    }

    (
        [min.x as f32, min.y as f32, min.z as f32],
        [max.x as f32, max.y as f32, max.z as f32],
    )
}

fn aabb_min(aabb: &crate::ray::aabb::Aabb) -> [f32; 3] {
    [aabb.x.min as f32, aabb.y.min as f32, aabb.z.min as f32]
}

fn aabb_max(aabb: &crate::ray::aabb::Aabb) -> [f32; 3] {
    [aabb.x.max as f32, aabb.y.max as f32, aabb.z.max as f32]
}
