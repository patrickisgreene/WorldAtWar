use bevy_ecs::prelude::*;
use bevy_math::DVec3;
use waw_geocoord::GeoCoord;

#[derive(Component)]
#[relationship(relationship_target = CityLabelFor)]
pub struct CityLabelEntity(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CityLabelEntity)]
pub struct CityLabelFor(Entity);

/// Marker component for a city entity
#[derive(Component, Debug, Clone)]
pub struct CityEntity {
    pub name: String,
    pub coordinate: GeoCoord,
    pub country_code: String,
}

/// Simple marker component to indicate a city has a label spawned
/// This is separate from the relationship to enable fast filtering
#[derive(Component, Debug)]
pub struct HasCityLabel;

/// Marker component for the UI label of a city
#[derive(Component, Debug)]
pub struct CityLabel {
    /// Reference to the city entity this label represents
    pub city_entity: Entity,
    /// World position of the city (cached for performance)
    pub world_position: DVec3,
}

/// Configuration for city label spawning/despawning
#[derive(Resource, Debug, Clone)]
pub struct CityLabelConfig {
    /// Distance threshold for spawning labels (in meters from camera)
    pub spawn_distance: f64,
    /// Distance threshold for despawning labels (in meters from camera)
    pub despawn_distance: f64,
    /// Font size for city labels
    pub font_size: f32,
}

impl Default for CityLabelConfig {
    fn default() -> Self {
        Self {
            // Spawn labels when camera is within 2 million meters (2000 km)
            spawn_distance: 1_000_000.0,
            // Despawn when camera moves beyond 2.5 million meters (2500 km)
            // Keep some hysteresis to prevent flickering, but not too much
            despawn_distance: 1_000_000.0,
            font_size: 16.0,
        }
    }
}