use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use waw_ron_asset::RonAssetPlugin;

use crate::{Combatant, CombatantColor};

pub struct CombatantsPlugin;

impl Plugin for CombatantsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<Combatant>::new("combatant.ron"))
            .add_systems(Startup, temp_load_combatants);
    }
}

fn temp_load_combatants(mut commands: Commands) {
    commands.spawn(Combatant {
        color: CombatantColor::Red,
        countries: vec!["US".into(), "CA".into()],
    });
    commands.spawn(Combatant {
        color: CombatantColor::Violet,
        countries: vec!["MX".into()],
    });
}
