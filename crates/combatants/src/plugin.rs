use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_asset::prelude::*;

use waw_ron_asset::RonAssetPlugin;

use crate::{Alliance, Alliances, Combatant, CombatantColor};

pub struct CombatantsPlugin;

impl Plugin for CombatantsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<Combatant>::new("combatant.ron"))
            .add_plugins(RonAssetPlugin::<Alliance>::new("alliance.ron"))
            .add_systems(Startup, temp_load_combatants)
            .add_systems(Startup, load_alliances);
    }
}

fn load_alliances(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Alliances(assets.load_folder("alliances")));
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
