use bevy_ecs::prelude::*;
use bevy_asset::prelude::*;
use bevy_reflect::prelude::*;
use serde::{Serialize, Deserialize};

use crate::CombatantColor;

#[derive(Asset, TypePath, Component, Serialize, Deserialize)]
pub struct Combatant {
    pub color: CombatantColor,
    pub countries: Vec<String>,
}
