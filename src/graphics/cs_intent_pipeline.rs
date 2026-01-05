use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayoutDescriptor,
    ShaderModule, ShaderStages,
};

pub struct CsIntentPipeline {
    pub pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl CsIntentPipeline {
    pub fn new(device: &Device, shader: &ShaderModule) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("CS Intent Bind Group Layout"),
            entries: &[
                // globals
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // grid
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                //intent
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("CS Intent Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("CS Intent Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: PipelineCompilationOptions::default(),
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn record(
        &self,
        pass: &mut ComputePass,
        bind_group: &CsIntentPipelineBindGroup,
        dispatch: (u32, u32, u32),
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group.bind_group, &[]);
        pass.dispatch_workgroups(dispatch.0, dispatch.1, dispatch.2);
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}

pub struct CsIntentPipelineBindGroup {
    pub bind_group: BindGroup,
}

impl CsIntentPipelineBindGroup {
    pub fn new(
        device: &Device,
        layout: &BindGroupLayout,
        globals: &Buffer,
        grid: &Buffer,
        intent: &Buffer,
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
                    resource: grid.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: intent.as_entire_binding(),
                },
            ],
        });

        Self { bind_group }
    }
}
