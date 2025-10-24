use bevy_asset::prelude::*;
use bevy_camera::visibility::Visibility;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_mesh::prelude::*;
use bevy_pbr::{MeshMaterial3d, wireframe::WireframeColor};
use bevy_transform::prelude::*;
use big_space::prelude::*;
use waw_utils::color::random_color;

use crate::{
    EarthMaterialHandle,
    detail::{CameraQuery, Leaf, UrbanEntityFor, distance_to_lod_level},
    geometry::{Chunk, EarthHeightMap, generate_chunk_mesh},
};

#[derive(Component, Default, Debug, PartialEq, Clone, Copy)]
pub struct Subdivided;

pub fn subdivide_chunks(
    camera: CameraQuery,
    mut commands: Commands,
    images: Res<Assets<Image>>,
    heightmap: Res<EarthHeightMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<EarthMaterialHandle>,
    chunks: Query<(Entity, &CellCoord, &Transform, &Chunk), Without<Subdivided>>,
) {
    let cam_world_pos = camera.camera_position();

    for (entity, chunk_coord, chunk_trans, chunk) in chunks.iter() {
        let chunk_world_pos = camera.world_position(chunk_coord, chunk_trans);
        let distance = cam_world_pos.distance(chunk_world_pos);
        let desired_level = distance_to_lod_level(distance);

        if desired_level > chunk.subdivision_level {
            // Hide the mesh and set the subdivided component.
            commands
                .entity(entity)
                .despawn_related::<UrbanEntityFor>()
                .insert((Visibility::Hidden, Subdivided));

            for child in chunk.get_children() {
                let (mesh, chunk_center) =
                    generate_chunk_mesh(&child, images.get(&heightmap.get()));

                // Calculate the center position of the chunk
                // in the global coordinate workspace.
                let (grid_entity, grid) = camera.full_grid();
                let (grid_cell, grid_offset) = grid.translation_to_grid(chunk_center);

                // Spawn the chunk and get its id.
                let child_id = commands
                    .grid(grid_entity, grid.clone())
                    .spawn_spatial((
                        child,
                        grid_cell,
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(material.get()),
                        Transform::from_translation(grid_offset),
                        WireframeColor {
                            color: random_color(),
                        },
                    ))
                    .id();

                // Add the graph relationship.
                commands.entity(entity).add_one_related::<Leaf>(child_id);
            }
        }
    }
}
