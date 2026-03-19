use std::sync::Arc;

pub struct GpuContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl GpuContext {
    pub fn new() -> Self {
        pollster::block_on(Self::init_async())
    }

    async fn init_async() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        println!("Using GPU: {}", adapter.get_info().name);

        let mut limits = wgpu::Limits::default();
        // limits.max_storage_buffer_binding_size = adapter.limits().max_storage_buffer_binding_size;
        // limits.max_buffer_size = adapter.limits().max_buffer_size;
        limits.max_storage_buffers_per_shader_stage = 12;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Raytracer Device"),
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::Performance,
                ..Default::default()
            })
            .await
            .expect("Failed to create GPU device");

        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
        }
    }
}
