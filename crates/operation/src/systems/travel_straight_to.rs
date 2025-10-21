use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_internal::log::warn;
use bevy_math::DVec3;
use bevy_time::prelude::*;
use bevy_image::prelude::*;
use bevy_transform::prelude::*;

use big_space::prelude::*;

use waw_geocoord::GeoCoord;
use waw_earth::EarthOriginGrid;
use waw_earth::geometry::EarthHeightMap;
use waw_weapons::data::{MovementSettings, Weapon};

use crate::*;

pub fn travel_straight_to(
    grid: Query<&Grid, With<EarthOriginGrid>>,
    time: Res<Time>,
    heightmap: Res<EarthHeightMap>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
    weapons: Res<Assets<Weapon>>,
    mut query: Query<(
        Entity,
        &Operation,
        &OperationIndex,
        &WeaponHandle,
        &mut TravelStraightToState,
        &mut CellCoord,
        &mut Transform,
    )>,
) {
    for (entity, operation, op_index, weapon, mut state, mut cell, mut trans) in query.iter_mut() {
        // Fetch the big_space grid.
        let Ok(grid) = grid.single() else {
            warn!("No Grid Present!");
            continue;
        };

        // Initialize start position on first frame
        if state.start.is_none() {
            let current_world_pos = grid.grid_position_double(&cell, &trans);
            // For some reason the current_world_pos is DVec3::ZERO witch causes a NaN
            // to appear in the Coord::from_world_pos calculation. to prevent this
            // early return and the position should be set on the next run after the
            // command queue is applied updateding the position.
            if current_world_pos == DVec3::ZERO {
                return;
            }
            let coord = GeoCoord::from_world(current_world_pos);
            state.start = Some(coord);
        }

        if let Some(weapon) = weapons.get(&**weapon) {
            let mut has_reached_location = false;
            match operation.maneuvers[**op_index] {
                // Entities should only pass the query filter if
                // they are currently in the `TravelTo` Maneuver
                // so the other variants are unreachable.
                Maneuver::Stop(_) | Maneuver::Release { .. } |
                Maneuver::BallisticTo(_) | Maneuver::Detonate => {}
                Maneuver::StraightTo(to) => {
                    if let Some(start) = state.start {
                        let altitude_offset = DVec3::new(0.0, weapon.movement.altitude, 0.0);
                        // Convert speed from km/h to m/s
                        let speed_kmh = weapon.movement.speed;
                        let speed_ms = speed_kmh * 1000.0 / 3600.0;

                        let start_pos = start.world_pos() + altitude_offset;
                        let end_pos = to.world_pos() + altitude_offset;

                        // Calculate total distance in meters
                        let total_distance = start_pos.distance(end_pos);

                        // If already at destination, mark as complete
                        if total_distance < 1.0 {
                            state.progress = 1.0;
                        } else {
                            // Calculate distance to travel this frame (in meters)
                            let distance_this_frame = speed_ms * time.delta_secs_f64();

                            // Update progress (0.0 to 1.0)
                            let progress_delta = distance_this_frame / total_distance;
                            state.progress += progress_delta;
                        }

                        // Check if we've reached the destination
                        if state.progress >= 1.0 {
                            state.progress = 1.0;

                            // Set final position
                            let (new_cell, local_translation) = grid.translation_to_grid(end_pos);
                            cell.set_if_neq(new_cell);
                            trans.translation = local_translation;

                            has_reached_location = true;
                        } else {
                            // Lerp between start and end position
                            let current_pos = start_pos.slerp(end_pos, state.progress as f64);

                            let height_map_height = if weapon.movement.settings.contains(&MovementSettings::FollowHeightMap) {
                                 sample_texture(images.get(&heightmap.get()), GeoCoord::from_world(current_pos).uv()) * 500000.0
                            } else {
                                0.0
                            };

                            // Add height map offset to the position
                            let height_offset = current_pos.normalize() * height_map_height as f64;
                            let adjusted_pos = current_pos + height_offset;

                            // First, get the base rotation to align with Earth's surface
                            let current_coord = GeoCoord::from_world(adjusted_pos);
                            let base_rotation = current_coord.rotation();

                            // Calculate direction of travel (in world space)
                            let forward = (end_pos - start_pos).normalize();
                            let up = adjusted_pos.normalize(); // Normal to Earth's surface

                            // Project forward direction onto the tangent plane (perpendicular to up)
                            let forward_tangent = (forward - up * forward.dot(up)).normalize();

                            // Calculate the heading rotation around the up axis
                            // This rotates from the base forward direction to our travel direction
                            let base_forward = base_rotation * bevy_math::Vec3::Z;
                            let base_forward_dvec = DVec3::new(
                                base_forward.x as f64,
                                base_forward.y as f64,
                                base_forward.z as f64
                            );

                            // Calculate angle between base forward and desired forward
                            let cos_angle = base_forward_dvec.dot(forward_tangent).clamp(-1.0, 1.0);
                            let angle = cos_angle.acos();

                            // Determine rotation direction using cross product
                            let cross = base_forward_dvec.cross(forward_tangent);
                            let rotation_dir = if cross.dot(up) > 0.0 { 1.0 } else { -1.0 };

                            // Create heading rotation around the up axis
                            let heading_rotation = bevy_math::Quat::from_axis_angle(
                                bevy_math::Vec3::new(up.x as f32, up.y as f32, up.z as f32),
                                (angle * rotation_dir) as f32
                            );

                            // Combine base rotation with heading rotation
                            trans.rotation = heading_rotation * base_rotation;

                            // Convert world position to grid coordinates
                            let (new_cell, local_translation) =
                                grid.translation_to_grid(adjusted_pos);
                            *cell = new_cell;
                            trans.translation = local_translation;
                        }
                    }
                }
            }

            if has_reached_location {
                commands
                    .entity(entity)
                    .remove::<TravelStraightTo>()
                    .remove::<TravelStraightToState>()
                    .insert(OperationIndex(**op_index + 1));
            }
        }
    }
}

fn sample_texture(image: Option<&Image>, uv: bevy_math::Vec2) -> f32 {
    if let Some(image) = image {
        let width = image.width() as f32;
        let height = image.height() as f32;

        // Wrap U coordinate for horizontal seam handling (repeat mode)
        let wrapped_u = uv.x.fract();

        // Convert UV coordinates to pixel coordinates
        let x = (wrapped_u * width).clamp(0.0, width - 1.0) as u32;
        let y = (uv.y * height).clamp(0.0, height - 1.0) as u32;

        if let Ok(color) = image.get_color_at(x, y) {
            return color.to_linear().red;
        }
    }
    0.0
}