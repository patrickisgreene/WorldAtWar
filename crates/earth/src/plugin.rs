use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_pbr::MaterialPlugin;

use crate::{material::EarthMaterial, systems};

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<EarthMaterial>::default(),))
            .add_systems(
                Update,
                (
                    systems::subdivide_chunks,
                    systems::merge_chunks.after(systems::subdivide_chunks),
                ),
            )
            .add_systems(Startup, systems::spawn_base_tiles);
    }
}
