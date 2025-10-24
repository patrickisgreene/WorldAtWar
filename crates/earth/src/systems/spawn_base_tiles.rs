use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_mesh::prelude::*;
use bevy_pbr::{prelude::*, wireframe::WireframeColor};
use bevy_transform::prelude::*;
use big_space::prelude::*;
use waw_utils::color::random_color;

use crate::{
    EarthMaterialHandle, EarthOriginGrid, EarthResolution,
    geometry::{Chunk, CubeFace, EarthHeightMap, EarthUrbanAreas, generate_chunk_mesh},
    material::EarthMaterial,
};

pub fn spawn_base_tiles(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    resolution: Option<Res<EarthResolution>>,
    grid: Query<(Entity, &Grid), With<EarthOriginGrid>>,
    mut materials: ResMut<Assets<EarthMaterial>>,
) {
    let res = resolution.map(|x| x.clone()).unwrap_or_default();
    commands.insert_resource(EarthHeightMap::create(&assets, res));
    commands.insert_resource(EarthUrbanAreas::create(&assets, res));

    let material = EarthMaterialHandle::create(&assets, &mut materials, res);

    for (grid_entity, grid) in &grid {
        for face in CubeFace::ALL {
            let chunk = Chunk::base(face);

            let (mesh, chunk_center) = generate_chunk_mesh(&chunk, None);

            let (grid_cell, grid_offset) = grid.translation_to_grid(chunk_center);

            commands.grid(grid_entity, grid.clone()).spawn_spatial((
                chunk,
                grid_cell,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.get()),
                Transform::from_translation(grid_offset),
                WireframeColor {
                    color: random_color(),
                },
            ));
        }
    }

    commands.insert_resource(material);
}
