use bevy_ecs::prelude::*;
use bevy_asset::prelude::*;

use crate::{labels::CityEntity, CountriesResource, data::Country};

/// System to spawn city entities from loaded country assets
/// This only spawns the city data entities, not the UI labels yet
pub fn spawn_city_entities(
    mut commands: Commands,
    countries_resource: Res<CountriesResource>,
    countries: Res<Assets<Country>>,
    loaded_folders: Res<Assets<bevy_asset::LoadedFolder>>,
    existing_cities: Query<&CityEntity>,
) {
    // Don't spawn if cities already exist
    if !existing_cities.is_empty() {
        return;
    }

    // Get the loaded folder
    let Some(folder) = loaded_folders.get(&countries_resource.0) else {
        return;
    };

    // Iterate through all country assets
    for handle in &folder.handles {
        let Ok(country_handle) = handle.clone().try_typed::<Country>() else {
            continue;
        };

        let Some(country) = countries.get(&country_handle) else {
            continue;
        };

        // Spawn an entity for each city
        for city in &country.cities {
            commands.spawn(CityEntity {
                name: city.name.clone(),
                coordinate: (&city.coordinates).into(),
                country_code: country.code.clone(),
            });
        }
    }
}
