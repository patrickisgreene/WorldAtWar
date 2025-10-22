use bevy_ecs::prelude::*;
use bevy_asset::prelude::*;

use crate::CountriesResource;

pub fn load_countries(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(CountriesResource(assets.load_folder("countries")));
}