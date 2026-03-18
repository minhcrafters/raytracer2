use wgpu::util::DeviceExt;

use crate::gpu::buffers::*;
use crate::gpu::context::GpuContext;
use crate::image::PPMImage;

pub struct SceneBuffers {
    pub camera: GpuCamera,
    pub materials: Vec<GpuMaterial>,
    pub transforms: Vec<GpuTransform>,
    pub spheres: Vec<GpuSphere>,
    pub quads: Vec<GpuQuad>,
    pub primitives: Vec<GpuPrimitive>,
    pub bvh_nodes: Vec<GpuBvhNode>,
    pub light_prim_indices: Vec<u32>,
    pub triangles: Vec<GpuTriangle>,
    pub hdri_pixels: Vec<[f32; 4]>,
    pub tex_pixels: Vec<[f32; 4]>,
}

pub struct GpuRenderer {
    ctx: GpuContext,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuRenderer {
    pub fn new() -> Self {
        let ctx = GpuContext::new();

        let shader_bytes = include_bytes!("../../shaders/raytracer.spv");
        let shader_module = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Raytracer Shader"),
                source: wgpu::util::make_spirv(shader_bytes),
            });

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Raytracer Bind Group Layout"),
                    entries: &Self::bind_group_layout_entries(),
                });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Raytracer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Raytracer Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            ctx,
            pipeline,
            bind_group_layout,
        }
    }

    fn bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = Vec::new();

        // Binding 0: Camera UBO
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        // Bindings 1-10: readonly storage buffers
        for i in 1..=10u32 {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: i,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        // Binding 11: Accum buffer (read-write)
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 11,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        // Binding 12: Output buffer (read-write)
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 12,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        entries
    }

    pub fn render(&self, scene: &SceneBuffers, spp: u32) -> PPMImage {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        let width = scene.camera.image_width;
        let height = scene.camera.image_height;
        let pixel_count = (width * height) as usize;

        // Create GPU buffers
        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&scene.camera),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let materials_buf = Self::create_storage_buffer(device, "Materials", &scene.materials);
        let transforms_buf = Self::create_storage_buffer(device, "Transforms", &scene.transforms);
        let spheres_buf = Self::create_storage_buffer_or_dummy(device, "Spheres", &scene.spheres);
        let quads_buf = Self::create_storage_buffer_or_dummy(device, "Quads", &scene.quads);
        let primitives_buf = Self::create_storage_buffer(device, "Primitives", &scene.primitives);
        let bvh_buf = Self::create_storage_buffer(device, "BVH", &scene.bvh_nodes);

        // Light buffer: [count, index0, index1, ...]
        let mut light_data = vec![scene.light_prim_indices.len() as u32];
        light_data.extend_from_slice(&scene.light_prim_indices);
        if light_data.len() < 2 {
            light_data.push(0); // dummy
        }
        let lights_buf = Self::create_storage_buffer(device, "Lights", &light_data);

        let triangles_buf =
            Self::create_storage_buffer_or_dummy(device, "Triangles", &scene.triangles);
        let hdri_buf = Self::create_storage_buffer_or_dummy(device, "HDRI", &scene.hdri_pixels);
        let tex_buf = Self::create_storage_buffer_or_dummy(device, "TexData", &scene.tex_pixels);

        // Accumulation buffer (zeroed)
        let accum_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Accum Buffer"),
            size: (pixel_count * 16) as u64, // vec4 per pixel
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Output buffer
        let output_size = (pixel_count * 16) as u64; // vec4 per pixel
        let output_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Staging buffer for readback
        let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Raytracer Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: materials_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: transforms_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: spheres_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: quads_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: primitives_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: bvh_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: lights_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: triangles_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: hdri_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: tex_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: accum_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: output_buf.as_entire_binding(),
                },
            ],
        });

        let workgroups_x = (width + 7) / 8;
        let workgroups_y = (height + 7) / 8;

        // One sample per dispatch
        for sample in 0..spp {
            let mut cam = scene.camera;
            cam.current_sample = sample;
            queue.write_buffer(&camera_buf, 0, bytemuck::bytes_of(&cam));

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Raytracer Pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
            }

            if sample == spp - 1 {
                encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
            }

            let submit_idx = queue.submit(std::iter::once(encoder.finish()));
            let _ = device.poll(wgpu::PollType::Wait {
                submission_index: Some(submit_idx),
                timeout: None,
            });

            print!("\rSample {}/{} ", sample + 1, spp);
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
        println!();

        let buffer_slice = staging_buf.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        receiver
            .recv()
            .unwrap()
            .expect("Failed to map staging buffer");

        let data = buffer_slice.get_mapped_range();
        let pixel_data: &[[f32; 4]] = bytemuck::cast_slice(&data);

        let mut image = PPMImage::new(width as usize, height as usize);
        for y in 0..height as usize {
            for x in 0..width as usize {
                let idx = y * width as usize + x;
                let px = pixel_data[idx];
                image.set_pixel_raw(
                    x,
                    y,
                    (px[0] * 255.0).clamp(0.0, 255.0) as u8,
                    (px[1] * 255.0).clamp(0.0, 255.0) as u8,
                    (px[2] * 255.0).clamp(0.0, 255.0) as u8,
                );
            }
        }

        drop(data);
        staging_buf.unmap();

        image
    }

    fn create_storage_buffer<T: bytemuck::Pod>(
        device: &wgpu::Device,
        label: &str,
        data: &[T],
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE,
        })
    }

    fn create_storage_buffer_or_dummy<T: bytemuck::Pod>(
        device: &wgpu::Device,
        label: &str,
        data: &[T],
    ) -> wgpu::Buffer {
        if data.is_empty() {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: &[0u8; 16],
                usage: wgpu::BufferUsages::STORAGE,
            })
        } else {
            Self::create_storage_buffer(device, label, data)
        }
    }
}
