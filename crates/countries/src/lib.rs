use bevy_asset::{LoadedFolder, prelude::*};
use bevy_ecs::prelude::*;

pub mod data;
pub mod labels;
mod plugin;
mod systems;

pub use plugin::CountriesPlugin;

#[derive(Resource)]
pub struct CountriesResource(pub Handle<LoadedFolder>);
