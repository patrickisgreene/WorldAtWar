use bevy_asset::AssetServer;
use bevy_camera::prelude::*;
use bevy_ecs::prelude::*;
use bevy_text::prelude::*;
use bevy_transform::prelude::*;
use bevy_ui::prelude::*;
use big_space::prelude::*;
use waw_combatants::{Combatant, CombatantColor};
use waw_earth::EarthLevelOfDetailFocus;

use crate::labels::{CityEntity, CityLabel, CityLabelConfig, CityLabelEntity, HasCityLabel};

/// System to manage label visibility based on camera distance
/// Spawns labels when cities are close enough, despawns when too far
pub fn manage_city_label_visibility(
    mut commands: Commands,
    assets: Res<AssetServer>,
    grid_query: Query<&Grid>,
    labels: Query<&CityLabel>,
    config: Res<CityLabelConfig>,
    combatants: Query<&Combatant>,
    cities_without_labels: Query<(Entity, &CityEntity), Without<HasCityLabel>>,
    cities_with_labels: Query<(Entity, &CityEntity, &CityLabelEntity), With<HasCityLabel>>,
    camera_query: Query<(&Transform, &CellCoord), (With<Camera3d>, With<EarthLevelOfDetailFocus>)>,
) {
    // Get camera position
    let Ok((camera_transform, camera_cell)) = camera_query.single() else {
        return;
    };

    // Get the grid to calculate world positions
    // For now, we'll assume there's only one grid
    let Ok(grid) = grid_query.single() else {
        return;
    };

    // Calculate camera world position
    let camera_world_pos = grid.grid_position_double(camera_cell, camera_transform);

    // Calculate camera altitude (distance from Earth's center minus Earth's radius)
    for (city_entity, city) in cities_without_labels.iter() {
        let city_world_pos = city.coordinate.world_pos();
        let distance = camera_world_pos.distance(city_world_pos);

        // Skip if too far
        if distance >= config.spawn_distance {
            continue;
        }

        // Check horizon visibility - only spawn labels for cities on the visible hemisphere
        let city_surface_normal = city_world_pos.normalize();
        let city_to_camera = (camera_world_pos - city_world_pos).normalize();
        let facing_camera = city_surface_normal.dot(city_to_camera);

        // If the city is facing away from camera (behind horizon), skip it
        if facing_camera <= 0.0 {
            continue;
        }

        let (bg, highlight) = combatants.into_iter()
            .filter(|c| c.countries.contains(&city.country_code))
            .map(|x|(x.color.background(), x.color.highlight()))
            .next()
            .unwrap_or((CombatantColor::Neutral.background(), CombatantColor::Neutral.highlight()));

        let container = commands
            .spawn((
                Name::new(format!("{} Label", city.name)),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    border: UiRect::all(px(2.0)),
                    padding: UiRect::all(px(5.0)),
                    ..Default::default()
                },
                BorderColor::all(highlight),
                BackgroundColor(bg.into()),
                BorderRadius::all(px(2.0)),
                CityLabel {
                    city_entity,
                    world_position: city_world_pos,
                },
            ))
            .id();

        let image_path = format!("textures/flags/icons/{}.png", city.country_code);

        let icon_entity = commands
            .spawn((
                ImageNode {
                    image: assets.load(image_path),
                    ..Default::default()
                },
                Node {
                    ..Default::default()
                },
            ))
            .id();

        // City is close enough and visible - spawn a label
        let label_entity = commands
            .spawn((
                Node {
                    padding: UiRect::all(px(4.0)),
                    border: UiRect::all(px(2.5)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Text::new(&city.name),
                TextFont {
                    font: assets.load("fonts/Monofur/MonofurNerdFontMono-Regular.ttf"),
                    font_size: config.font_size,
                    ..Default::default()
                },
                TextLayout::new_with_justify(Justify::Center),
                TextColor::WHITE,
            ))
            .id();

        commands
            .entity(container)
            .add_children(&[icon_entity, label_entity]);

        // Mark the city as having a label (both marker and direct component)
        // Also add the radar gizmo visibility marker
        commands
            .entity(city_entity)
            .insert(HasCityLabel)
            .insert(CityLabelEntity(container))
            .insert(waw_radar::RadarGizmoVisible);
    }

    // Check existing labels - despawn if too far OR behind horizon
    for (city_entity, city, has_label) in cities_with_labels.iter() {
        // Get the label entity to verify it still exists
        let Ok(_label) = labels.get(has_label.0) else {
            // Label doesn't exist anymore, remove the marker
            commands.entity(city_entity).remove::<HasCityLabel>();
            continue;
        };

        // Calculate fresh distance from camera to city (don't use cached label.world_position)
        let city_world_pos = city.coordinate.world_pos();
        let distance = camera_world_pos.distance(city_world_pos);

        // Check if city is too far
        let too_far = distance > config.despawn_distance;

        // Check if city is behind the horizon
        let city_surface_normal = city_world_pos.normalize();
        let city_to_camera = (camera_world_pos - city_world_pos).normalize();
        let facing_camera = city_surface_normal.dot(city_to_camera);
        let behind_horizon = facing_camera <= 0.0;

        // Despawn if too far OR behind horizon
        if too_far || behind_horizon {
            // Despawn the label entity directly
            commands.entity(has_label.0).despawn();

            // Remove the marker, relationship, and radar gizmo visibility from the city entity
            commands
                .entity(city_entity)
                .remove::<HasCityLabel>()
                .remove::<CityLabelEntity>()
                .remove::<waw_radar::RadarGizmoVisible>();
        }
    }
}
