use bevy_camera::prelude::*;
use bevy_ecs::prelude::*;
use bevy_transform::prelude::*;
use bevy_ui::prelude::*;
use big_space::prelude::*;
use waw_earth::EarthLevelOfDetailFocus;

use crate::labels::CityLabel;

/// System to update UI label positions based on camera movement
/// Projects 3D world positions to 2D screen positions
pub fn update_city_label_positions(
    grid_query: Query<(Entity, &Grid)>,
    mut labels: Query<(&mut Node, &CityLabel, &mut Visibility)>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &CellCoord, &Transform),
        (With<Camera3d>, With<EarthLevelOfDetailFocus>),
    >,
) {
    let Ok((camera, camera_global_transform, camera_cell, camera_transform)) =
        camera_query.single()
    else {
        return;
    };

    let Ok((_grid_entity, grid)) = grid_query.single() else {
        return;
    };

    // Calculate camera world position for horizon check
    let camera_world_pos = grid.grid_position_double(camera_cell, camera_transform);

    for (mut node, label, mut visibility) in labels.iter_mut() {
        // Calculate the surface normal at the city (points outward from Earth's center)
        let city_surface_normal = label.world_position.normalize();

        // Calculate the vector from city to camera
        let city_to_camera = (camera_world_pos - label.world_position).normalize();

        // Check if the city is facing the camera using dot product
        // If the city's surface normal points toward the camera, it's on the visible side
        let facing_camera = city_surface_normal.dot(city_to_camera);

        // If facing_camera <= 0, the city is on the far side of the horizon, hide it
        if facing_camera <= 0.0 {
            *visibility = Visibility::Hidden;
            continue;
        }

        // Convert the city's world position to grid coordinates
        let (city_cell, city_offset) = grid.translation_to_grid(label.world_position);

        // Calculate the cell difference between city and camera
        let cell_diff = city_cell - *camera_cell;

        // Calculate the world-space position relative to camera's cell
        // In big_space, we work with positions relative to the camera's grid cell
        let cell_offset_in_world = cell_diff.as_dvec3(grid) * grid.cell_edge_length() as f64;
        let city_pos_relative_to_camera_cell = cell_offset_in_world + city_offset.as_dvec3();

        // Now convert to local camera space by subtracting camera's local position
        let pos_in_camera_cell =
            city_pos_relative_to_camera_cell - camera_transform.translation.as_dvec3();

        // Convert to f32 Vec3 for projection
        let local_pos = pos_in_camera_cell.as_vec3();

        // Project to screen space using the camera's world_to_viewport
        // The GlobalTransform already accounts for the camera's orientation and position
        if let Ok(viewport_pos) = camera.world_to_viewport(camera_global_transform, local_pos) {
            // Update the UI position
            node.left = Val::Px(viewport_pos.x);
            node.top = Val::Px(viewport_pos.y);
            *visibility = Visibility::Inherited;
        } else {
            // If projection fails (e.g., behind camera), hide the label
            *visibility = Visibility::Hidden;
        }
    }
}
