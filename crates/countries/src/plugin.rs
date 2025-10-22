use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use waw_ron_asset::RonAssetPlugin;

use crate::{systems, data::Country, labels::CityLabelConfig};

pub struct CountriesPlugin;

impl Plugin for CountriesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<Country>::new("country.ron"))
            .init_resource::<CityLabelConfig>()
            .add_systems(Startup, systems::load_countries)
            .add_systems(
                Update,
                (
                    systems::spawn_city_entities,
                    systems::manage_city_label_visibility.after(systems::spawn_city_entities),
                    systems::update_city_label_positions.after(systems::manage_city_label_visibility),
                ),
            );
    }
}