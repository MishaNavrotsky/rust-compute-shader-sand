use std::{
    borrow::Cow,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use wgpu::{
    Adapter, BindGroup, BindGroupLayout, Buffer, BufferBinding, BufferDescriptor, BufferView,
    Color, CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, DeviceDescriptor, ExperimentalFeatures, Features,
    FragmentState, Instance, Limits, LoadOp, MemoryHints, Operations, PipelineCompilationOptions,
    PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp,
    Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureFormat, TextureView,
    TextureViewDescriptor, VertexState, util::DeviceExt,
};
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};

const CS_OUTPUT_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub async fn create_graphics(window: Arc<Window>, proxy: EventLoopProxy<Graphics>) {
    let instance = Instance::default();
    let surface = instance.create_surface(Arc::clone(&window)).unwrap();

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Could not get an adapter (GPU).");

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: None,
            required_features: Features::empty(),
            required_limits: Limits::defaults().using_resolution(adapter.limits()),
            memory_hints: MemoryHints::Performance,
            trace: Default::default(),
            experimental_features: ExperimentalFeatures::default(),
        })
        .await
        .expect("Failed to get device");

    let size = window.inner_size();
    let width = size.width.max(1);
    let height = size.height.max(1);
    let surface_config = surface.get_default_config(&adapter, width, height).unwrap();

    surface.configure(&device, &surface_config);

    let globals_bind = GlobalsBind {
        buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals Buffer"),
            contents: bytemuck::bytes_of(&Globals::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }),
    };
    let pipelines = prepare_pipelines(&device, &surface_config);
    let targets = prepare_targets(
        &device,
        &queue,
        &surface_config,
        &pipelines.cs_bind_group_layout,
        &pipelines.rs_bind_group_layout,
        &globals_bind.buffer,
    );
    let gfx = Graphics {
        window: window.clone(),
        instance,
        surface,
        surface_config,
        adapter,
        device,
        queue,
        pipelines,
        targets,
        globals: globals_bind,
        start_instant: Instant::now(),

        mouse_pos: [0.0, 0.0],

        ping: true,
    };

    let _ = proxy.send_event(gfx);
}

pub fn prepare_pipelines(device: &Device, surface_config: &SurfaceConfiguration) -> Pipelines {
    let globals_bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<Globals>() as u64),
        },
        count: None,
    };
    let compute_buffer_a_bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let compute_buffer_b_bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
        binding: 3,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let cs_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute BGL"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: CS_OUTPUT_TEXTURE_FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            globals_bind_group_layout_entry.clone(),
            compute_buffer_a_bind_group_layout_entry.clone(),
            compute_buffer_b_bind_group_layout_entry.clone(),
        ],
    });
    let compute_pipeline = create_compute_pipeline(&device, &cs_bind_group_layout);

    let rs_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Render BGL"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    format: CS_OUTPUT_TEXTURE_FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            globals_bind_group_layout_entry.clone(),
        ],
    });
    let render_pipeline =
        create_render_pipeline(&device, &rs_bind_group_layout, surface_config.format);

    Pipelines {
        cs_bind_group_layout,
        rs_bind_group_layout,
        compute_pipeline,
        render_pipeline,
    }
}

fn prepare_targets(
    device: &Device,
    queue: &Queue,
    surface_config: &SurfaceConfiguration,
    cs_bind_group_layout: &BindGroupLayout,
    rs_bind_group_layout: &BindGroupLayout,
    globals_buffer: &Buffer,
) -> Targets {
    let compute_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Compute Output Texture"),
        size: wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CS_OUTPUT_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let compute_texture_view = compute_texture.create_view(&TextureViewDescriptor::default());

    let cell_count = surface_config.width as u64 * surface_config.height as u64;
    let buffer_size = cell_count * std::mem::size_of::<u32>() as u64;
    let compute_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Compute Buffer A"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let compute_buffer_a_bind_group_entry = wgpu::BindGroupEntry {
        binding: 2,
        resource: compute_buffer_a.as_entire_binding(),
    };

    let compute_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Compute Buffer B"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let compute_buffer_b_bind_group_entry = wgpu::BindGroupEntry {
        binding: 3,
        resource: compute_buffer_b.as_entire_binding(),
    };

    let globals_bind_group_entry = wgpu::BindGroupEntry {
        binding: 1,
        resource: globals_buffer.as_entire_binding(),
    };

    let cs_bind_group_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compute Bind Group AB"),
        layout: &cs_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&compute_texture_view),
            },
            globals_bind_group_entry.clone(),
            wgpu::BindGroupEntry {
                binding: 2,
                ..compute_buffer_a_bind_group_entry.clone()
            },
            wgpu::BindGroupEntry {
                binding: 3,
                ..compute_buffer_b_bind_group_entry.clone()
            },
        ],
    });
    let cs_bind_group_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compute Bind Group BA"),
        layout: &cs_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&compute_texture_view),
            },
            globals_bind_group_entry.clone(),
            wgpu::BindGroupEntry {
                binding: 2,
                ..compute_buffer_b_bind_group_entry.clone()
            },
            wgpu::BindGroupEntry {
                binding: 3,
                ..compute_buffer_a_bind_group_entry.clone()
            },
        ],
    });
    let rs_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Render Bind Group"),
        layout: &rs_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&compute_texture_view),
            },
            globals_bind_group_entry.clone(),
        ],
    });

    let zero_data = vec![0u32; cell_count as usize];
    queue.write_buffer(&compute_buffer_a, 0, bytemuck::cast_slice(&zero_data));
    queue.write_buffer(&compute_buffer_b, 0, bytemuck::cast_slice(&zero_data));

    Targets {
        compute_buffer_a,
        compute_buffer_b,
        compute_texture,
        compute_texture_view,
        cs_bind_group_ab,
        cs_bind_group_ba,
        rs_bind_group,
    }
}

fn create_compute_pipeline(device: &Device, bgl: &wgpu::BindGroupLayout) -> ComputePipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute_shader.wgsl"))),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[bgl],
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&ComputePipelineDescriptor {
        cache: None,
        label: Some("Compute Pipeline"),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: PipelineCompilationOptions::default(),
    })
}

fn create_render_pipeline(
    device: &Device,
    bgl: &wgpu::BindGroupLayout,
    swap_chain_format: TextureFormat,
) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("render_shader.wgsl"))),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(swap_chain_format.into())],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    })
}

#[derive(Debug)]
pub struct GlobalsBind {
    buffer: Buffer,
}

#[derive(Debug)]
pub struct Pipelines {
    cs_bind_group_layout: BindGroupLayout,
    rs_bind_group_layout: BindGroupLayout,

    compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
}

#[derive(Debug)]
pub struct Targets {
    compute_buffer_a: Buffer,
    compute_buffer_b: Buffer,
    compute_texture: Texture,
    compute_texture_view: TextureView,
    cs_bind_group_ab: BindGroup,
    cs_bind_group_ba: BindGroup,
    rs_bind_group: BindGroup,
}

#[derive(Debug)]
pub struct Graphics {
    window: Arc<Window>,
    instance: Instance,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    pipelines: Pipelines,
    targets: Targets,
    globals: GlobalsBind,

    start_instant: Instant,

    mouse_pos: [f32; 2],
    ping: bool,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Globals {
    pub resolution: [f32; 2], // width, height
    pub time: f32,
    pub _pad0: f32, // padding to 16 bytes

    pub mouse_pos: [f32; 2], // x, y
    pub _pad1: [f32; 2],     // padding
}

impl Graphics {
    pub fn set_mouse_pos(&mut self, mouse_pos: [f32; 2]) {
        self.mouse_pos = mouse_pos;
    }
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        self.targets = prepare_targets(
            &self.device,
            &self.queue,
            &self.surface_config,
            &self.pipelines.cs_bind_group_layout,
            &self.pipelines.rs_bind_group_layout,
            &self.globals.buffer,
        );
    }

    pub fn run_cs(&mut self, command_encoder: &mut CommandEncoder) {
        let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Compute pass"),
            ..Default::default()
        });

        compute_pass.set_pipeline(&self.pipelines.compute_pipeline);

        let bind_group = if self.ping {
            &self.targets.cs_bind_group_ab
        } else {
            &self.targets.cs_bind_group_ba
        };
        compute_pass.set_bind_group(0, bind_group, &[]);

        let x = (self.surface_config.width + 7) / 8;
        let y = (self.surface_config.height + 7) / 8;

        compute_pass.dispatch_workgroups(x, y, 1);

        self.ping = !self.ping;
    }

    pub fn run_rs(&mut self, command_encoder: &mut CommandEncoder, frame: &mut SurfaceTexture) {
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut r_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        r_pass.set_pipeline(&self.pipelines.render_pipeline);
        r_pass.set_bind_group(0, &self.targets.rs_bind_group, &[]);
        r_pass.draw(0..3, 0..1);
    }

    pub fn draw(&mut self) {
        let mut frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture.");

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let globals = Globals {
            resolution: [
                self.surface_config.width as f32,
                self.surface_config.height as f32,
            ],
            time: Instant::now()
                .duration_since(self.start_instant)
                .as_secs_f32(),
            mouse_pos: self.mouse_pos,
            ..Globals::default()
        };

        self.queue
            .write_buffer(&self.globals.buffer, 0, bytemuck::bytes_of(&globals));
        self.run_cs(&mut encoder);
        self.run_rs(&mut encoder, &mut frame);

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
