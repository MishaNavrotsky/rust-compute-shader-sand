use bytemuck::Zeroable;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device, Extent3d, Queue, SurfaceConfiguration, Texture,
    TextureDescriptor, TextureDimension, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::graphics::structures::{Globals, GridCell, Intent};

pub struct SimulationStateResources {
    pub grid: Buffer,
    pub grid_next: Buffer,
    pub intent: Buffer,

    width: usize,
    height: usize,
}

pub struct SimulationConstResources {
    pub globals: Buffer,
}

impl SimulationStateResources {
    pub fn new(device: &Device, surface_configuration: &SurfaceConfiguration) -> Self {
        let width = surface_configuration.width as usize;
        let height = surface_configuration.height as usize;
        let cell_count = width * height;

        let grid_size = (cell_count * std::mem::size_of::<GridCell>()) as u64;
        let intent_size = (cell_count * std::mem::size_of::<Intent>()) as u64;

        let grid = device.create_buffer(&BufferDescriptor {
            label: Some("Grid Buffer"),
            size: grid_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let grid_next = device.create_buffer(&BufferDescriptor {
            label: Some("Grid Next Buffer"),
            size: grid_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let intent = device.create_buffer(&BufferDescriptor {
            label: Some("Intent Buffer"),
            size: intent_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            grid,
            grid_next,
            intent,
            width,
            height,
        }
    }

    pub fn zero_initialize(&self, queue: &Queue) {
        let cell_count = self.width * self.height;

        let zero_grid = vec![GridCell::zeroed(); cell_count];
        let zero_intent = vec![Intent::zeroed(); cell_count];

        queue.write_buffer(&self.grid, 0, bytemuck::cast_slice(&zero_grid));
        queue.write_buffer(&self.grid_next, 0, bytemuck::cast_slice(&zero_grid));
        queue.write_buffer(&self.intent, 0, bytemuck::cast_slice(&zero_intent));
    }
}

impl SimulationConstResources {
    pub fn new(device: &Device) -> Self {
        let globals = device.create_buffer(&BufferDescriptor {
            label: Some("Simulation Globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { globals }
    }

    pub fn zero_initialize_globals(&self, queue: &Queue) {
        queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&Globals::default()));
    }

    pub fn update_globals(&self, queue: &Queue, globals: &Globals) {
        queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(globals));
    }
}
