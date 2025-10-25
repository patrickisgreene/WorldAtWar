use bevy_asset::prelude::*;
use bevy_color::palettes::tailwind::LIME_400;
use bevy_ecs::prelude::*;
use bevy_transform::prelude::*;
use big_space::prelude::*;
use waw_geocoord::GeoCoord;

use crate::{CountriesResource, data::Country, labels::CityEntity};
use waw_earth::EarthOriginGrid;

/// System to spawn city entities from loaded country assets
/// This only spawns the city data entities, not the UI labels yet
pub fn spawn_city_entities(
    mut commands: Commands,
    countries_resource: Res<CountriesResource>,
    countries: Res<Assets<Country>>,
    loaded_folders: Res<Assets<bevy_asset::LoadedFolder>>,
    existing_cities: Query<&CityEntity>,
    earth_grid: Query<(Entity, &Grid), With<EarthOriginGrid>>,
) {
    // Don't spawn if cities already exist
    if !existing_cities.is_empty() {
        return;
    }

    // Get the loaded folder
    let Some(folder) = loaded_folders.get(&countries_resource.0) else {
        return;
    };

    // Get the Earth grid for positioning
    let Ok((grid_entity, grid)) = earth_grid.single() else {
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
            let coordinate: GeoCoord = (&city.coordinates).into();

            // Convert geographic coordinate to world position
            let world_pos = coordinate.world_pos();

            // Convert world position to grid cell and offset
            let (grid_cell, grid_offset) = grid.translation_to_grid(world_pos);

            // Spawn the city entity in the big_space hierarchy
            commands.grid(grid_entity, grid.clone()).spawn_spatial((
                grid_cell,
                CityEntity {
                    name: city.name.clone(),
                    coordinate,
                    country_code: country.code.clone(),
                },
                waw_radar::RadarGizmoColor(LIME_400.into()),
                waw_radar::RadarShape::Sphere { radius: 70000.0 },
                waw_radar::Radar {
                    strength: 1.0,
                    rotation: Default::default(),
                    translation: Default::default(),
                },
                waw_weapons::Inventory::default(),
                Transform::from_translation(grid_offset),
            ));
        }
    }
}
