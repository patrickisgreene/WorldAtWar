use bevy::{
    asset::RenderAssetUsages,
    ecs::{lifecycle::HookContext, query::QueryItem, world::DeferredWorld},
    prelude::*,
    render::{
        extract_component::ExtractComponent, gpu_readback::Readback, render_resource::*,
        storage::ShaderStorageBuffer,
    },
};
use big_space::prelude::*;

mod pass;
mod pipeline;
mod plugin;
mod systems;

pub use plugin::EarthPickingPlugin;

pub fn picking_hook(mut world: DeferredWorld, context: HookContext) {
    let mut buffers = world.resource_mut::<Assets<ShaderStorageBuffer>>();
    let mut buffer = ShaderStorageBuffer::with_size(
        GpuPickingData::min_size().get() as usize,
        RenderAssetUsages::default(),
    );
    buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    let buffer = buffers.add(buffer);

    world
        .commands()
        .entity(context.entity)
        .insert(Readback::buffer(buffer.clone()))
        .observe(systems::picking_readback);

    let mut picking_data = world.get_mut::<PickingData>(context.entity).unwrap();
    picking_data.buffer = buffer;
}

#[derive(Default, Clone, Component)]
#[component(on_add = picking_hook)]
pub struct PickingData {
    pub cursor_coords: Vec2,
    pub cell: CellCoord,           // cell of floating origin (camera)
    pub translation: Option<Vec3>, // relative to floating origin cell
    pub world_from_clip: Mat4,
    buffer: Handle<ShaderStorageBuffer>,
}

impl ExtractComponent for PickingData {
    type QueryData = &'static PickingData;
    type QueryFilter = ();
    type Out = GpuPickingBuffer;

    fn extract_component(data: QueryItem<'_, '_, Self::QueryData>) -> Option<Self::Out> {
        Some(GpuPickingBuffer(data.buffer.id()))
    }
}

#[derive(Component)]
pub struct GpuPickingBuffer(AssetId<ShaderStorageBuffer>);

#[derive(Default, Debug, Clone, ShaderType)]
pub struct GpuPickingData {
    pub cursor_coords: Vec2,
    pub depth: f32,
    pub stencil: u32,
    pub world_from_clip: Mat4,
    pub cell: IVec3,
}
