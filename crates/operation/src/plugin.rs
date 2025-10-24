use bevy_app::prelude::*;
use bevy_hanabi::HanabiPlugin;

use crate::systems;

pub struct OperationsPlugin;

impl Plugin for OperationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(Update, systems::initialize_weapon_instances)
            .add_systems(
                Update,
                (
                    systems::orbit_around,
                    systems::ballistic_to,
                    systems::travel_straight_to,
                    systems::maneuver_index_updated,
                    systems::cleanup_detonation,
                ),
            );
    }
}
