mod color;
mod plugin;
mod combatant;

pub use combatant::Combatant;
pub use color::CombatantColor;
pub use plugin::CombatantsPlugin;

use bevy_ecs::prelude::*;
use bevy_asset::{prelude::*, LoadedFolder};
use bevy_reflect::TypePath;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum CombatantRelationship {
    Friendly,
    Enemy
}

#[derive(Resource)]
pub struct Alliances(pub Handle<LoadedFolder>);

#[derive(Asset, TypePath, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alliance {
    pub name_short: String,
    pub name_long: Option<String>,
    pub color: (f32, f32, f32),
    pub countries: Vec<String>
}