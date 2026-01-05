use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};

use std::{sync::Arc, time::Instant};

use winit::window::Window;

use crate::graphics::{
    cs_intent_pipeline::{CsIntentPipeline, CsIntentPipelineBindGroup},
    cs_resolve_pipeline::{CsResolvePipeline, CsResolvePipelineBindGroup},
    resources::{SimulationConstResources, SimulationStateResources},
    rs_pipeline::{RsPipeline, RsPipelineBindGroup},
};

#[derive(Debug, Clone, Copy)]
pub enum CursorState {
    Default,
    Pressed,
}

pub struct FrameResources {
    pub cs_intent_pipeline_bind_group: CsIntentPipelineBindGroup,
    pub cs_resolve_pipeline_bind_group: CsResolvePipelineBindGroup,
    pub rs_pipeline_bind_group: RsPipelineBindGroup,
}

pub struct Graphics {
    pub window: Arc<Window>,
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,

    pub resources_state: SimulationStateResources,
    pub resources_const: SimulationConstResources,

    pub cs_intent_pipeline: CsIntentPipeline,
    pub cs_resolve_pipeline: CsResolvePipeline,

    pub rs_pipeline: RsPipeline,

    pub frames: [FrameResources; 2],

    pub start_instant: Instant,
    pub mouse_pos: [f32; 2],
    pub cursor_state: CursorState,
    pub frame_index: usize,
}

// struct Globals {
//   resolution    : vec2<f32>,
//   mouse_pos     : vec2<f32>,

//   time          : f32,
//   cursor_state  : f32,
//   _pad0         : vec2<f32>,
// };
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct Globals {
    pub resolution: [f32; 2], // width, height
    pub mouse_pos: [f32; 2],  // x, y

    pub time: f32,
    pub mouse_state: f32,
    pub _pad0: [f32; 2], // padding
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Empty = 0,
    Sand = 1,
}
// struct GridCell {
//   color : vec4<f32>,
//   state : u32,
//   flags : u32,
//   _pad  : vec2<u32>,
// };
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridCell {
    pub color: [f32; 4],
    pub state: u32, //CellState
    pub flags: u32,
    pub _pad: [u32; 2],
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentKind {
    Stay = 0,
    MoveDown = 1,
    MoveDL = 2,
    MoveDR = 3,
}
// struct Intent {
//   kind  : u32,
//   flags : u32,
// };
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Intent {
    pub kind: u32, // MoveKind
    pub flags: u32,
}
