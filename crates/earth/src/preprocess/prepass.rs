use crate::{
    earth::EarthComponents,
    data::{AttachmentFormat, GpuTileAtlas},
};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{binding_types::*, *},
        renderer::{RenderContext, RenderDevice},
    },
};

pub(crate) fn create_mip_layout(
    device: &RenderDevice,
    format: AttachmentFormat,
) -> BindGroupLayout {
    device.create_bind_group_layout(
        None,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<u32>(false), // atlas_index
                texture_2d_array(TextureSampleType::Float { filterable: true }), // parent
                texture_storage_2d_array(
                    format.processing_format(),
                    StorageTextureAccess::WriteOnly,
                ), // child
            ),
        ),
    )
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MipPrepass;

impl render_graph::Node for MipPrepass {
    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let gpu_tile_atlases = world.resource::<EarthComponents<GpuTileAtlas>>();

        context.add_command_buffer_generation_task(move |device| {
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            for gpu_tile_atlas in gpu_tile_atlases.values() {
                gpu_tile_atlas.generate_mip(&mut pass, pipeline_cache);
            }

            drop(pass);

            encoder.finish()
        });

        Ok(())
    }
}
