use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            BindGroupEntries, CommandEncoderDescriptor, ComputePassDescriptor, PipelineCache,
        },
        renderer::RenderContext,
        storage::GpuShaderStorageBuffer,
    },
};

use crate::{
    picking::{GpuPickingBuffer, pipeline::PickingPipeline},
    render::EarthViewDepthTexture,
};

#[derive(Debug, Hash, Default, PartialEq, Eq, Clone, RenderLabel)]
pub struct PickingPass;

impl ViewNode for PickingPass {
    type ViewQuery = (&'static GpuPickingBuffer, &'static EarthViewDepthTexture);

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        context: &mut RenderContext<'w>,
        (picking_buffer, depth): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let picking_pipeline = world.resource::<PickingPipeline>();
        let buffer = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();

        let Some(pipeline) = pipeline_cache.get_compute_pipeline(picking_pipeline.id) else {
            return Ok(());
        };

        let Some(buffer) = buffer.get(picking_buffer.0) else {
            return Ok(());
        };

        let bind_group = context.render_device().create_bind_group(
            None,
            &picking_pipeline.layout,
            &BindGroupEntries::sequential((
                buffer.buffer.as_entire_binding(),
                &depth.depth_view,
                &depth.stencil_view,
            )),
        );

        context.add_command_buffer_generation_task(move |device| {
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(1, 1, 1);
            drop(pass);

            encoder.finish()
        });

        Ok(())
    }
}
