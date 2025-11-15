use crate::{
    data::{AttachmentLabel, GpuTileAtlas, TileAtlas, TileTree, finish_loading, start_loading},
    earth::{EarthComponents, EarthConfig},
    material::EarthMaterial,
    preprocess::{MipPipelines, MipPrepass},
    render::{
        DepthCopyPipeline, EarthItem, EarthMaterialPlugin, EarthPass, EarthTilingPrepassPipelines,
        GpuEarth, GpuEarthView, TilingPrepass, TilingPrepassItem, extract_earth_phases,
        prepare_earth_depth_textures, queue_tiling_prepass,
    },
    shaders::{InternalShaders, load_earth_shaders},
    utils::TiffLoader,
    view::EarthViewComponents,
};
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        graph::CameraDriverLabel,
        render_graph::{RenderGraph, RenderGraphExt, ViewNodeRunner},
        render_phase::{DrawFunctions, ViewSortedRenderPhases, sort_phase_system},
        render_resource::*,
    },
};
use bevy_common_assets::ron::RonAssetPlugin;
use big_space::prelude::*;

#[derive(Resource)]
pub struct EarthSettings {
    pub attachments: Vec<AttachmentLabel>,
    pub atlas_size: u32,
}

impl Default for EarthSettings {
    fn default() -> Self {
        Self {
            attachments: vec![
                AttachmentLabel::Topography,
                AttachmentLabel::DayTime,
                AttachmentLabel::NightTime,
                AttachmentLabel::OceanMask,
                AttachmentLabel::Bathyometry
            ],
            atlas_size: 1028,
        }
    }
}

/// The plugin for the earth renderer.
pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigSpaceDefaultPlugins);

        app.add_plugins(RonAssetPlugin::<EarthConfig>::new(&["tc.ron"]))
            .init_asset::<EarthConfig>()
            .init_resource::<InternalShaders>()
            .init_resource::<EarthViewComponents<TileTree>>()
            .init_resource::<EarthSettings>()
            .init_asset_loader::<TiffLoader>()
            .add_systems(
                PostUpdate,
                (
                    // Todo: enable visibility checking again
                    // check_visibility::<With<TileAtlas>>.in_set(VisibilitySystems::CheckVisibility),
                    (
                        TileTree::compute_requests,
                        finish_loading,
                        TileAtlas::update,
                        start_loading,
                        TileTree::adjust_to_tile_atlas,
                        TileTree::generate_surface_approximation,
                        TileTree::update_earth_view_buffer,
                        TileAtlas::update_earth_buffer,
                    )
                        .chain()
                        .after(TransformSystems::Propagate),
                ),
            );
        app.sub_app_mut(RenderApp)
            .init_resource::<SpecializedComputePipelines<MipPipelines>>()
            .init_resource::<SpecializedComputePipelines<EarthTilingPrepassPipelines>>()
            .init_resource::<EarthComponents<GpuTileAtlas>>()
            .init_resource::<EarthComponents<GpuEarth>>()
            .init_resource::<EarthViewComponents<GpuEarthView>>()
            .init_resource::<EarthViewComponents<TilingPrepassItem>>()
            .init_resource::<DrawFunctions<EarthItem>>()
            .init_resource::<ViewSortedRenderPhases<EarthItem>>()
            .add_systems(
                ExtractSchedule,
                (
                    extract_earth_phases,
                    GpuTileAtlas::initialize,
                    GpuTileAtlas::extract.after(GpuTileAtlas::initialize),
                    GpuEarth::initialize.after(GpuTileAtlas::initialize),
                    GpuEarthView::initialize,
                ),
            )
            .add_systems(
                Render,
                (
                    (
                        GpuTileAtlas::prepare,
                        GpuEarth::prepare,
                        GpuEarthView::prepare_earth_view,
                        GpuEarthView::prepare_indirect,
                        GpuEarthView::prepare_refine_tiles,
                    )
                        .in_set(RenderSystems::Prepare),
                    sort_phase_system::<EarthItem>.in_set(RenderSystems::PhaseSort),
                    prepare_earth_depth_textures.in_set(RenderSystems::PrepareResources),
                    (queue_tiling_prepass, GpuTileAtlas::queue).in_set(RenderSystems::Queue),
                    GpuTileAtlas::_cleanup
                        .before(World::clear_entities)
                        .in_set(RenderSystems::Cleanup),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<EarthPass>>(Core3d, EarthPass)
            .add_render_graph_edges(
                Core3d,
                (Node3d::StartMainPass, EarthPass, Node3d::MainOpaquePass),
            );

        let mut render_graph = app
            .sub_app_mut(RenderApp)
            .world_mut()
            .resource_mut::<RenderGraph>();
        render_graph.add_node(MipPrepass, MipPrepass);
        render_graph.add_node(TilingPrepass, TilingPrepass);
        render_graph.add_node_edge(MipPrepass, TilingPrepass);
        render_graph.add_node_edge(TilingPrepass, CameraDriverLabel);

    app.add_plugins(EarthMaterialPlugin::<EarthMaterial>::default());
    }

    fn finish(&self, app: &mut App) {
        let attachments = app.world().resource::<EarthSettings>().attachments.clone();

        load_earth_shaders(app, &attachments);

        app.sub_app_mut(RenderApp)
            .init_resource::<EarthTilingPrepassPipelines>()
            .init_resource::<MipPipelines>()
            .init_resource::<DepthCopyPipeline>();
    }
}
