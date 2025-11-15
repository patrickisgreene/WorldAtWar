use crate::{
    debug::DebugEarth,
    earth::EarthComponents,
    data::GpuTileAtlas,
    render::{
        DrawEarthCommand, EARTH_DEPTH_FORMAT, EarthItem, EarthTilingPrepassPipelines, GpuEarthView,
        SetEarthBindGroup, SetEarthViewBindGroup,
    },
    shaders::{DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER},
    utils::{EarthsToSpawn, spawn_earths},
    view::EarthViewComponents,
};
use bevy::{
    pbr::{MeshPipeline, MeshPipelineViewLayoutKey, SetMaterialBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, SetItemPipeline,
            ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::RetainedViewEntity,
    },
    shader::{ShaderDefVal, ShaderRef},
};
use std::{hash::Hash, marker::PhantomData};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct EarthPipelineKey {
    pub flags: EarthPipelineFlags,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct EarthPipelineFlags: u32 {
        const NONE               = 0;
        const WIREFRAME          = 1 <<  1;
        const SHOW_DATA_LOD      = 1 <<  2;
        const SHOW_GEOMETRY_LOD  = 1 <<  3;
        const SHOW_TILE_TREE     = 1 <<  4;
        const SHOW_PIXELS        = 1 <<  5;
        const SHOW_UV            = 1 <<  6;
        const SHOW_NORMALS       = 1 <<  7;
        const MORPH              = 1 <<  8;
        const BLEND              = 1 <<  9;
        const TILE_TREE_LOD      = 1 << 10;
        const SAMPLE_GRAD        = 1 << 12;
        const HIGH_PRECISION     = 1 << 13;
        const MSAA_RESERVED_BITS = EarthPipelineFlags::MSAA_MASK_BITS << EarthPipelineFlags::MSAA_SHIFT_BITS;
    }
}

impl EarthPipelineFlags {
    const MSAA_MASK_BITS: u32 = 0b111111;
    const MSAA_SHIFT_BITS: u32 = 32 - 6;

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits = ((msaa_samples - 1) & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        EarthPipelineFlags::from_bits(msaa_bits).unwrap()
    }

    pub fn from_debug(debug: &DebugEarth) -> Self {
        let mut key = EarthPipelineFlags::NONE;

        if debug.wireframe {
            key |= EarthPipelineFlags::WIREFRAME;
        }
        if debug.show_data_lod {
            key |= EarthPipelineFlags::SHOW_DATA_LOD;
        }
        if debug.show_geometry_lod {
            key |= EarthPipelineFlags::SHOW_GEOMETRY_LOD;
        }
        if debug.show_tile_tree {
            key |= EarthPipelineFlags::SHOW_TILE_TREE;
        }
        if debug.show_pixels {
            key |= EarthPipelineFlags::SHOW_PIXELS;
        }
        if debug.show_uv {
            key |= EarthPipelineFlags::SHOW_UV;
        }
        if debug.show_normals {
            key |= EarthPipelineFlags::SHOW_NORMALS;
        }
        if debug.morph {
            key |= EarthPipelineFlags::MORPH;
        }
        if debug.blend {
            key |= EarthPipelineFlags::BLEND;
        }
        if debug.tile_tree_lod {
            key |= EarthPipelineFlags::TILE_TREE_LOD;
        }
        if debug.sample_grad {
            key |= EarthPipelineFlags::SAMPLE_GRAD;
        }
        if debug.high_precision {
            key |= EarthPipelineFlags::HIGH_PRECISION;
        }

        key
    }

    pub fn msaa_samples(&self) -> u32 {
        ((self.bits() >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS) + 1
    }

    pub fn polygon_mode(&self) -> PolygonMode {
        match self.contains(EarthPipelineFlags::WIREFRAME) {
            true => PolygonMode::Line,
            false => PolygonMode::Fill,
        }
    }

    pub fn shader_defs(&self) -> Vec<ShaderDefVal> {
        let mut shader_defs = Vec::new();

        if self.contains(EarthPipelineFlags::SHOW_DATA_LOD) {
            shader_defs.push("SHOW_DATA_LOD".into());
        }
        if self.contains(EarthPipelineFlags::SHOW_GEOMETRY_LOD) {
            shader_defs.push("SHOW_GEOMETRY_LOD".into());
        }
        if self.contains(EarthPipelineFlags::SHOW_TILE_TREE) {
            shader_defs.push("SHOW_TILE_TREE".into());
        }
        if self.contains(EarthPipelineFlags::SHOW_PIXELS) {
            shader_defs.push("SHOW_PIXELS".into())
        }
        if self.contains(EarthPipelineFlags::SHOW_UV) {
            shader_defs.push("SHOW_UV".into());
        }
        if self.contains(EarthPipelineFlags::SHOW_NORMALS) {
            shader_defs.push("SHOW_NORMALS".into())
        }
        if self.contains(EarthPipelineFlags::MORPH) {
            shader_defs.push("MORPH".into());
        }
        if self.contains(EarthPipelineFlags::BLEND) {
            shader_defs.push("BLEND".into());
        }
        if self.contains(EarthPipelineFlags::TILE_TREE_LOD) {
            shader_defs.push("TILE_TREE_LOD".into());
        }
        if self.contains(EarthPipelineFlags::SAMPLE_GRAD) {
            shader_defs.push("SAMPLE_GRAD".into());
        }

        if self.contains(EarthPipelineFlags::HIGH_PRECISION) {
            shader_defs.push("HIGH_PRECISION".into());
        }

        shader_defs
    }
}

/// The pipeline used to render the earth entities.
#[derive(Resource)]
pub struct EarthRenderPipeline<M: Material> {
    view_layout: BindGroupLayout,
    view_layout_multisampled: BindGroupLayout,
    earth_layout: BindGroupLayout,
    earth_view_layout: BindGroupLayout,
    material_layout: BindGroupLayout,
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
    marker: PhantomData<M>,
}

impl<M: Material> FromWorld for EarthRenderPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let mesh_pipeline = world.resource::<MeshPipeline>();
        let prepass_pipelines = world.resource::<EarthTilingPrepassPipelines>();

        let vertex_shader = match M::vertex_shader() {
            ShaderRef::Default => world.load_asset(DEFAULT_VERTEX_SHADER),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => world.load_asset(path),
        };

        let fragment_shader = match M::fragment_shader() {
            ShaderRef::Default => world.load_asset(DEFAULT_FRAGMENT_SHADER),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => world.load_asset(path),
        };

        Self {
            view_layout: mesh_pipeline
                .get_view_layout(MeshPipelineViewLayoutKey::empty())
                .main_layout
                .clone(),
            view_layout_multisampled: mesh_pipeline
                .get_view_layout(MeshPipelineViewLayoutKey::MULTISAMPLED)
                .main_layout
                .clone(),
            earth_layout: prepass_pipelines.earth_layout.clone(),
            earth_view_layout: prepass_pipelines.earth_view_layout.clone(),
            material_layout: M::bind_group_layout(device),
            vertex_shader,
            fragment_shader,
            marker: PhantomData,
        }
    }
}

impl<M: Material> SpecializedRenderPipeline for EarthRenderPipeline<M> {
    type Key = EarthPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = key.flags.shader_defs();

        let mut bind_group_layout = match key.flags.msaa_samples() {
            1 => vec![self.view_layout.clone()],
            _ => {
                shader_defs.push("MULTISAMPLED".into());
                vec![self.view_layout_multisampled.clone()]
            }
        };

        bind_group_layout.push(self.earth_layout.clone());
        bind_group_layout.push(self.earth_view_layout.clone());
        bind_group_layout.push(self.material_layout.clone());

        let mut vertex_shader_defs = shader_defs.clone();
        vertex_shader_defs.push("VERTEX".into());
        let mut fragment_shader_defs = shader_defs.clone();
        fragment_shader_defs.push("FRAGMENT".into());

        RenderPipelineDescriptor {
            label: None,
            layout: bind_group_layout,
            push_constant_ranges: default(),
            vertex: VertexState {
                shader: self.vertex_shader.clone(),
                entry_point: Some("vertex".into()),
                shader_defs: vertex_shader_defs,
                buffers: Vec::new(),
            },
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: key.flags.polygon_mode(),
                conservative: false,
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
            },
            fragment: Some(FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: fragment_shader_defs,
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(DepthStencilState {
                format: EARTH_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState {
                        compare: CompareFunction::GreaterEqual,
                        fail_op: StencilOperation::Keep,
                        depth_fail_op: StencilOperation::Keep,
                        pass_op: StencilOperation::Replace,
                    },
                    back: StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: key.flags.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

/// The draw function of the earth. It sets the pipeline and the bind groups and then issues the
/// draw call.
pub(crate) type DrawEarth = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetEarthBindGroup<1>,
    SetEarthViewBindGroup<2>,
    SetMaterialBindGroup<3>,
    DrawEarthCommand,
);

/// Queses all earth entities for rendering via the earth pipeline.
#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_earth<M: Material>(
    draw_functions: Res<DrawFunctions<EarthItem>>,
    debug: Option<Res<DebugEarth>>,
    pipeline_cache: Res<PipelineCache>,
    earth_pipeline: Res<EarthRenderPipeline<M>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<EarthRenderPipeline<M>>>,
    mut earth_phases: ResMut<ViewSortedRenderPhases<EarthItem>>,
    gpu_tile_atlases: Res<EarthComponents<GpuTileAtlas>>,
    gpu_earth_views: Res<EarthViewComponents<GpuEarthView>>,
    mut views: Query<(MainEntity, &Msaa)>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    let draw_function = draw_functions.read().get_id::<DrawEarth>().unwrap();

    for (view, msaa) in &mut views {
        let Some(earth_phase) = earth_phases.get_mut(&RetainedViewEntity {
            main_entity: view.into(),
            auxiliary_entity: Entity::PLACEHOLDER.into(),
            subview_index: 0,
        }) else {
            continue;
        };

        for (&earth, _gpu_tile_atlas) in gpu_tile_atlases.iter() {
            let Some(gpu_earth_view) = gpu_earth_views.get(&(earth, view)) else {
                continue;
            };

            let mut flags = EarthPipelineFlags::from_msaa_samples(msaa.samples());

            if let Some(debug) = &debug {
                flags |= EarthPipelineFlags::from_debug(debug);
            } else {
                flags |= EarthPipelineFlags::SAMPLE_GRAD;
            }

            let key = EarthPipelineKey { flags };

            let pipeline = pipelines.specialize(&pipeline_cache, &earth_pipeline, key);

            earth_phase.add(EarthItem {
                representative_entity: (earth, earth.into()), // technically wrong
                draw_function,
                pipeline,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                order: gpu_earth_view.order,
            })
        }
    }
}

/// This plugin adds a custom material for earth.
///
/// It can be used to render the earth using a custom vertex and fragment shader.
pub struct EarthMaterialPlugin<M: Material>(PhantomData<M>);

impl<M: Material> Default for EarthMaterialPlugin<M> {
    fn default() -> Self {
        Self(default())
    }
}

impl<M: Material + Clone> Plugin for EarthMaterialPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<M>::default())
            .insert_resource(EarthsToSpawn::<M>(vec![]))
            .add_systems(PostUpdate, spawn_earths::<M>);

        app.sub_app_mut(RenderApp)
            .add_render_command::<EarthItem, DrawEarth>()
            .init_resource::<SpecializedRenderPipelines<EarthRenderPipeline<M>>>()
            .add_systems(Render, queue_earth::<M>.in_set(RenderSystems::QueueMeshes));
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<EarthRenderPipeline<M>>();
    }
}
