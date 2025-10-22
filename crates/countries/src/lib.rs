use bevy_ecs::prelude::*;
use bevy_asset::{prelude::*, LoadedFolder};


mod plugin;
mod systems;
pub mod data;
pub mod labels;

pub use plugin::CountriesPlugin;

#[derive(Resource)]
pub struct CountriesResource(pub Handle<LoadedFolder>);