use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, Device, FragmentState,
    PipelineLayoutDescriptor, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderStages, TextureFormat, VertexState,
};

pub struct RsPipeline {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
}

impl RsPipeline {
    pub fn new(device: &Device, shader: &ShaderModule, target_format: TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("RS Bind Group Layout"),
            entries: &[
                // globals
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // grid_next
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("RS Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("RS Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(target_format.into())],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn record(&self, pass: &mut RenderPass, bind_group: &RsPipelineBindGroup) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}

pub struct RsPipelineBindGroup {
    pub bind_group: BindGroup,
}

impl RsPipelineBindGroup {
    pub fn new(
        device: &Device,
        layout: &BindGroupLayout,
        globals: &Buffer,
        grid_next: &Buffer,
    ) -> Self {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("intent_bind_group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: grid_next.as_entire_binding(),
                },
            ],
        });

        Self { bind_group }
    }
}
