use bevy::{
    core_pipeline::core_3d::graph::Core3d,
    prelude::*,
    render::{
        RenderApp,
        extract_component::ExtractComponentPlugin,
        render_graph::{RenderGraphExt, ViewNodeRunner},
    },
};

use crate::{
    picking::{PickingData, pass::PickingPass, pipeline::PickingPipeline, systems},
    render::EarthPass,
};

pub struct EarthPickingPlugin;

impl Plugin for EarthPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            systems::picking_system.after(TransformSystems::Propagate),
        )
        .add_plugins(ExtractComponentPlugin::<PickingData>::default());

        app.sub_app_mut(RenderApp)
            .add_render_graph_node::<ViewNodeRunner<PickingPass>>(Core3d, PickingPass)
            .add_render_graph_edge(Core3d, EarthPass, PickingPass);
    }
    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<PickingPipeline>();
    }
}
