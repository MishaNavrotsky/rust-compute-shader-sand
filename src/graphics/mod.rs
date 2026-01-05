use std::{borrow::Cow, sync::Arc, time::Instant};

use wgpu::{
    Color, CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, DeviceDescriptor,
    ExperimentalFeatures, Features, Instance, LoadOp, MemoryHints, Operations, PowerPreference,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, ShaderModuleDescriptor,
    ShaderSource, StoreOp, SurfaceTexture, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};

use crate::graphics::{
    cs_intent_pipeline::{CsIntentPipeline, CsIntentPipelineBindGroup},
    cs_resolve_pipeline::{CsResolvePipeline, CsResolvePipelineBindGroup},
    resources::{SimulationConstResources, SimulationStateResources},
    rs_pipeline::{RsPipeline, RsPipelineBindGroup},
    structures::FrameResources,
};

mod cs_intent_pipeline;
mod cs_resolve_pipeline;
mod resources;
mod rs_pipeline;
pub mod structures;

pub async fn create_graphics(window: Arc<Window>, proxy: EventLoopProxy<structures::Graphics>) {
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
            required_limits: adapter.limits(),
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

    let cs_intent_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Compute Shader Intent"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../shaders/compute_shader_intent.wgsl"
        ))),
    });

    let cs_resolve_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Compute Shader Resolve"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../shaders/compute_shader_resolve.wgsl"
        ))),
    });

    let rs_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/render_shader.wgsl"))),
    });

    let resources_state = create_resources_state(&device, &queue, &surface_config);
    let resources_const = SimulationConstResources::new(&device);
    resources_const.zero_initialize_globals(&queue);

    let cs_intent_pipeline = CsIntentPipeline::new(&device, &cs_intent_shader);
    let cs_resolve_pipeline = CsResolvePipeline::new(&device, &cs_resolve_shader);
    let rs_pipeline = RsPipeline::new(&device, &rs_shader, surface_config.format);

    let frames = create_frame_resources(
        &device,
        &resources_state,
        &resources_const,
        &cs_intent_pipeline,
        &cs_resolve_pipeline,
        &rs_pipeline,
    );

    let gfx = structures::Graphics {
        window: window.clone(),
        instance,
        surface,
        surface_config,
        adapter,
        device,
        queue,
        start_instant: Instant::now(),

        resources_state,
        resources_const,

        cs_intent_pipeline,
        cs_resolve_pipeline,
        rs_pipeline,

        frames,

        mouse_pos: [0.0, 0.0],
        cursor_state: structures::CursorState::Default,

        frame_index: 0,
    };

    let _ = proxy.send_event(gfx);
}

fn create_resources_state(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_config: &wgpu::SurfaceConfiguration,
) -> SimulationStateResources {
    let resources_state = SimulationStateResources::new(device, surface_config);
    resources_state.zero_initialize(queue);
    resources_state
}

fn create_frame_resources(
    device: &wgpu::Device,
    resources_state: &SimulationStateResources,
    resources_const: &SimulationConstResources,
    cs_intent_pipeline: &CsIntentPipeline,
    cs_resolve_pipeline: &CsResolvePipeline,
    rs_pipeline: &RsPipeline,
) -> [FrameResources; 2] {
    let cs_intent_pipeline_bind_group_a = CsIntentPipelineBindGroup::new(
        device,
        cs_intent_pipeline.bind_group_layout(),
        &resources_const.globals,
        &resources_state.grid,
        &resources_state.intent,
    );
    let cs_intent_pipeline_bind_group_b = CsIntentPipelineBindGroup::new(
        device,
        cs_intent_pipeline.bind_group_layout(),
        &resources_const.globals,
        &resources_state.grid_next,
        &resources_state.intent,
    );

    let cs_resolve_pipeline_bind_group_ab = CsResolvePipelineBindGroup::new(
        device,
        cs_resolve_pipeline.bind_group_layout(),
        &resources_const.globals,
        &resources_state.grid,
        &resources_state.grid_next,
        &resources_state.intent,
    );
    let cs_resolve_pipeline_bind_group_ba = CsResolvePipelineBindGroup::new(
        device,
        cs_resolve_pipeline.bind_group_layout(),
        &resources_const.globals,
        &resources_state.grid_next,
        &resources_state.grid,
        &resources_state.intent,
    );

    let rs_pipeline_bind_group_a = RsPipelineBindGroup::new(
        device,
        &rs_pipeline.bind_group_layout(),
        &resources_const.globals,
        &resources_state.grid,
    );
    let rs_pipeline_bind_group_b = RsPipelineBindGroup::new(
        device,
        rs_pipeline.bind_group_layout(),
        &resources_const.globals,
        &resources_state.grid_next,
    );

    let frames = [
        FrameResources {
            cs_intent_pipeline_bind_group: cs_intent_pipeline_bind_group_a,
            cs_resolve_pipeline_bind_group: cs_resolve_pipeline_bind_group_ab,
            rs_pipeline_bind_group: rs_pipeline_bind_group_a,
        },
        FrameResources {
            cs_intent_pipeline_bind_group: cs_intent_pipeline_bind_group_b,
            cs_resolve_pipeline_bind_group: cs_resolve_pipeline_bind_group_ba,
            rs_pipeline_bind_group: rs_pipeline_bind_group_b,
        },
    ];
    frames
}

impl structures::Graphics {
    fn get_frame_resources(&self) -> &FrameResources {
        &self.frames[self.frame_index % 2]
    }
    pub fn set_mouse_pos(&mut self, mouse_pos: [f32; 2]) {
        self.mouse_pos = mouse_pos;
    }
    pub fn set_cursor_state(&mut self, cursor_state: structures::CursorState) {
        self.cursor_state = cursor_state;
    }
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);

        self.resources_state =
            create_resources_state(&self.device, &self.queue, &self.surface_config);

        self.frames = create_frame_resources(
            &self.device,
            &self.resources_state,
            &self.resources_const,
            &self.cs_intent_pipeline,
            &self.cs_resolve_pipeline,
            &self.rs_pipeline,
        );
    }

    pub fn run_cs(&self, command_encoder: &mut CommandEncoder) {
        let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Compute pass"),
            ..Default::default()
        });

        let x = (self.surface_config.width + 7) / 8;
        let y = (self.surface_config.height + 7) / 8;

        self.cs_intent_pipeline.record(
            &mut compute_pass,
            &self.get_frame_resources().cs_intent_pipeline_bind_group,
            (x, y, 1),
        );

        self.cs_resolve_pipeline.record(
            &mut compute_pass,
            &self.get_frame_resources().cs_resolve_pipeline_bind_group,
            (x, y, 1),
        );
    }

    pub fn run_rs(&self, command_encoder: &mut CommandEncoder, frame: &mut SurfaceTexture) {
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

        self.rs_pipeline.record(
            &mut r_pass,
            &self.get_frame_resources().rs_pipeline_bind_group,
        );
    }

    pub fn draw(&mut self) {
        let mut frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture.");

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let globals = structures::Globals {
            resolution: [self.surface_config.width, self.surface_config.height],
            time: Instant::now()
                .duration_since(self.start_instant)
                .as_secs_f32(),
            mouse_pos: self.mouse_pos,
            mouse_state: self.cursor_state as u32,
            ..structures::Globals::default()
        };
        self.resources_const.update_globals(&self.queue, &globals);

        self.run_cs(&mut encoder);
        self.run_rs(&mut encoder, &mut frame);

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.frame_index ^= 1;
    }
}
