use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_image::prelude::*;
use bevy_internal::log::warn;
use bevy_math::DVec3;
use bevy_time::prelude::*;
use bevy_transform::prelude::*;

use big_space::prelude::*;
use waw_earth::geometry::EarthHeightMap;
use waw_weapons::data::{MovementSettings, Weapon};

use waw_geocoord::GeoCoord;

use crate::{
    Maneuver, Operation, OperationIndex, OrbitAround, OrbitAroundState, OrbitLength, StopBehavior,
    WeaponHandle,
};

pub fn orbit_around(
    grids: Grids,
    time: Res<Time>,
    mut commands: Commands,
    weapons: Res<Assets<Weapon>>,
    heightmap: Res<EarthHeightMap>,
    images: Res<Assets<Image>>,
    mut query: Query<(
        Entity,
        &Operation,
        &OperationIndex,
        &WeaponHandle,
        &mut OrbitAroundState,
        &mut CellCoord,
        &mut Transform,
    )>,
) {
    for (entity, operation, op_index, weapon, mut state, mut cell, mut trans) in query.iter_mut() {
        // Fetch the big_space grid.
        let Some(grid) = grids.parent_grid(entity) else {
            warn!("Entity {:?} has no parent grid", entity);
            continue;
        };

        if let Some(weapon) = weapons.get(&**weapon) {
            match operation.maneuvers[**op_index] {
                // Entities should only pass the query filter if
                // they are currently in the `Orbit` Maneuver
                // so the other variants are unreachable.
                Maneuver::Release { .. }
                | Maneuver::StraightTo(_)
                | Maneuver::Stop(StopBehavior::Stop)
                | Maneuver::BallisticTo(_)
                | Maneuver::Detonate => unreachable!(),
                Maneuver::Stop(StopBehavior::Orbit {
                    center,
                    radius,
                    length,
                }) => {
                    let altitude_offset = DVec3::new(0.0, weapon.movement.altitude, 0.0);

                    // Convert speed from km/h to m/s
                    let speed_kmh = weapon.movement.speed;
                    let speed_ms = speed_kmh * 1000.0 / 3600.0;

                    // Calculate the center position with altitude
                    let center_pos = center.world_pos() + altitude_offset;

                    // Calculate the circumference of the orbit circle
                    let circumference = 2.0 * std::f64::consts::PI * radius;

                    // Calculate distance to travel this frame (in meters)
                    let distance_this_frame = speed_ms * time.delta_secs_f64();

                    // Update progress (normalized to 0.0 to 1.0 for one complete orbit)
                    let progress_delta = distance_this_frame / circumference;
                    state.progress += progress_delta;

                    // Calculate current angle (in radians) based on progress
                    // progress wraps around: 0.0 = start, 1.0 = one full orbit
                    let angle = state.progress * 2.0 * std::f64::consts::PI;

                    // Calculate position on the circle
                    // We need to find a tangent plane at the center point to create the orbit
                    let center_normal = center_pos.normalize();

                    // Create two orthogonal vectors on the tangent plane
                    // Use an arbitrary up vector, then create the tangent basis
                    let arbitrary_up = if center_normal.y.abs() < 0.9 {
                        DVec3::Y
                    } else {
                        DVec3::X
                    };
                    let tangent1 = center_normal.cross(arbitrary_up).normalize();
                    let tangent2 = center_normal.cross(tangent1).normalize();

                    // Calculate current position on the orbit circle
                    let offset =
                        tangent1 * (angle.cos() * radius) + tangent2 * (angle.sin() * radius);
                    let current_pos = center_pos + offset;

                    // Apply height map offset if enabled
                    let height_map_height = if weapon
                        .movement
                        .settings
                        .contains(&MovementSettings::FollowHeightMap)
                    {
                        sample_texture(
                            images.get(&heightmap.get()),
                            GeoCoord::from_world(current_pos).uv(),
                        )
                    } else {
                        0.0
                    };

                    // Add height map offset to the position
                    let height_offset = current_pos.normalize() * height_map_height as f64;
                    let adjusted_pos = current_pos + height_offset;

                    // Calculate the rotation for the vehicle
                    // Base rotation aligned with Earth's surface
                    let current_coord = GeoCoord::from_world(adjusted_pos);
                    let base_rotation = current_coord.rotation();

                    // Calculate direction of travel (tangent to the circle)
                    let forward = tangent1 * (-angle.sin()) + tangent2 * (angle.cos());
                    let forward_normalized = forward.normalize();
                    let up = adjusted_pos.normalize(); // Normal to Earth's surface

                    // Project forward direction onto the tangent plane
                    let forward_tangent =
                        (forward_normalized - up * forward_normalized.dot(up)).normalize();

                    // Calculate the heading rotation
                    let base_forward = base_rotation * bevy_math::Vec3::Z;
                    let base_forward_dvec = DVec3::new(
                        base_forward.x as f64,
                        base_forward.y as f64,
                        base_forward.z as f64,
                    );

                    let cos_angle = base_forward_dvec.dot(forward_tangent).clamp(-1.0, 1.0);
                    let heading_angle = cos_angle.acos();

                    let cross = base_forward_dvec.cross(forward_tangent);
                    let rotation_dir = if cross.dot(up) > 0.0 { 1.0 } else { -1.0 };

                    let heading_rotation = bevy_math::Quat::from_axis_angle(
                        bevy_math::Vec3::new(up.x as f32, up.y as f32, up.z as f32),
                        (heading_angle * rotation_dir) as f32,
                    );

                    trans.rotation = heading_rotation * base_rotation;

                    // Convert world position to grid coordinates
                    let (new_cell, local_translation) = grid.translation_to_grid(adjusted_pos);
                    *cell = new_cell;
                    trans.translation = local_translation;

                    // Check if orbit is complete based on OrbitLength
                    match length {
                        OrbitLength::Count(count) => {
                            if state.progress >= count as f64 {
                                // Orbit complete, move to next maneuver
                                commands
                                    .entity(entity)
                                    .remove::<OrbitAround>()
                                    .remove::<OrbitAroundState>()
                                    .insert(OperationIndex(**op_index + 1));
                            }
                        }
                        OrbitLength::Indefinite => {
                            // Never complete, continue orbiting indefinitely
                        }
                    }
                }
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
