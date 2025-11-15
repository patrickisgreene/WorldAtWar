use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderStages, TextureSampleType,
            binding_types::{
                storage_buffer, texture_2d_multisampled, texture_depth_2d_multisampled,
            },
        },
        renderer::RenderDevice,
    },
};

use crate::{picking::GpuPickingData, shaders::PICKING_SHADER};

#[derive(Resource)]
pub struct PickingPipeline {
    pub id: CachedComputePipelineId,
    pub layout: BindGroupLayout,
}

impl FromWorld for PickingPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let layout = device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer::<GpuPickingData>(false),
                    texture_depth_2d_multisampled(),
                    texture_2d_multisampled(TextureSampleType::Uint),
                ),
            ),
        );

        let id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: world.load_asset(PICKING_SHADER),
            shader_defs: vec![],
            entry_point: Some("pick".into()),
            zero_initialize_workgroup_memory: false,
        });

        Self { id, layout }
    }
}
