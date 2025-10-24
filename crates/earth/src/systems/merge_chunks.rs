use bevy_camera::visibility::Visibility;
use bevy_ecs::prelude::*;
use bevy_mesh::prelude::*;
use bevy_platform::collections::HashMap;
use bevy_transform::prelude::*;
use big_space::prelude::*;

use crate::{
    detail::{
        Branch, CHILDREN_PER_CHUNK, CameraQuery, Leaf, UrbanEntityFor, distance_to_lod_level,
    },
    geometry::Chunk,
};

pub fn merge_chunks(
    mut commands: Commands,
    camera: CameraQuery,
    chunks: Query<(Entity, &CellCoord, &Transform, &Chunk, Option<&Leaf>), With<Mesh3d>>,
) {
    let cam_world_pos = camera.camera_position();

    let mut wants_merge: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (entity, chunk_coord, chunk_trans, chunk, leaf) in chunks.iter() {
        let chunk_world_pos = camera.world_position(chunk_coord, chunk_trans);

        let distance = cam_world_pos.distance(chunk_world_pos);

        let desired_level = distance_to_lod_level(distance);

        if desired_level < chunk.subdivision_level && leaf.is_some() {
            let parent = leaf.unwrap().0;
            if wants_merge.contains_key(&parent) {
                wants_merge.get_mut(&parent).unwrap().push(entity);
            } else {
                wants_merge.insert(parent, vec![entity]);
            }
        }
    }

    for (parent, children) in wants_merge {
        if children.len() == CHILDREN_PER_CHUNK {
            commands
                .entity(parent)
                .insert(Visibility::Visible)
                .remove::<super::subdivide_chunks::Subdivided>()
                .despawn_related::<UrbanEntityFor>()
                .despawn_related::<Branch>();
        }
    }
}
