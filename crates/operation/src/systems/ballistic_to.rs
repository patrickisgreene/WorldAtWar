use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_internal::log::warn;
use bevy_math::{DVec3, Mat3, Quat};
use bevy_time::prelude::*;
use bevy_transform::prelude::*;

use big_space::prelude::*;
use waw_weapons::data::Weapon;

use waw_geocoord::GeoCoord;

use crate::{BallisticTo, BallisticToState, Maneuver, Operation, OperationIndex, WeaponHandle};

pub fn ballistic_to(
    grids: Grids,
    time: Res<Time>,
    mut commands: Commands,
    weapons: Res<Assets<Weapon>>,
    mut query: Query<(
        Entity,
        &Operation,
        &OperationIndex,
        &WeaponHandle,
        &mut BallisticToState,
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
            match operation.maneuvers[**op_index] {
                // Entities should only pass the query filter if
                // they are currently in the `Orbit` Maneuver
                // so the other variants are unreachable.
                Maneuver::Stop(_) | Maneuver::StraightTo(_) |
                Maneuver::Detonate | Maneuver::Release { .. } => unreachable!(),
                Maneuver::BallisticTo(to) => {
                    let start = state.start.unwrap();
                    let delta_time = time.delta_secs_f64();

                    // Calculate the angular distance traveled this frame based on weapon speed
                    // Speed is in m/s, we need to convert to angular progress (0.0 to 1.0)
                    let start_pos = start.world_pos();
                    let end_pos = to.world_pos();

                    // Calculate great circle distance in meters
                    let distance = great_circle_distance(start_pos, end_pos);

                    // Progress increment per second
                    let progress_per_second = weapon.movement.speed / distance;
                    state.progress += progress_per_second * delta_time;

                    // Clamp progress to [0, 1]
                    state.progress = state.progress.clamp(0.0, 1.0);

                    // Calculate the current position along the ballistic arc
                    let current_pos = ballistic_arc_position(
                        start_pos,
                        end_pos,
                        state.progress,
                        distance,
                    );

                    // Update entity position using big_space grid system
                    //grid.set_grid_position_double(&mut cell, &mut trans, current_pos);
                    let (new_cell, new_trans) = grid.translation_to_grid(current_pos);
                    *cell = new_cell;
                    trans.translation = new_trans;

                    // Calculate rotation to face direction of travel
                    if state.progress < 1.0 {
                        // Calculate tangent direction for rotation
                        let next_t = (state.progress + 0.01).min(1.0);
                        let next_pos = ballistic_arc_position(start_pos, end_pos, next_t, distance);
                        let direction = (next_pos - current_pos).normalize();

                        // Orient the weapon to face the direction of travel
                        if direction.length_squared() > 0.001 {
                            let up = current_pos.normalize();
                            let forward = direction;
                            let right = forward.cross(up).normalize();
                            let adjusted_up = right.cross(forward);

                            trans.rotation = Quat::from_mat3(&Mat3::from_cols(
                                right.as_vec3(),
                                adjusted_up.as_vec3(),
                                -forward.as_vec3(),
                            ));
                        }
                    }

                    // Check if maneuver is complete
                    if state.progress >= 1.0 {
                        // Maneuver complete - advance to next maneuver or despawn
                        // Remove ballistic component to exit this system
                        commands.entity(entity)
                            .remove::<BallisticToState>()
                            .remove::<BallisticTo>()
                            .insert(OperationIndex(op_index.0 + 1));
                    }
                }
            }
        }
    }
}

/// Calculate great circle distance between two points on a sphere
fn great_circle_distance(start: DVec3, end: DVec3) -> f64 {
    let start_normalized = start.normalize();
    let end_normalized = end.normalize();

    // Use the dot product to find the angle between the vectors
    let dot = start_normalized.dot(end_normalized).clamp(-1.0, 1.0);
    let angle = dot.acos();

    // Multiply by Earth's radius to get actual distance
    let earth_radius = 6_371_000.0;
    angle * earth_radius
}

/// Calculate position along a ballistic arc between two points
/// This creates a realistic ICBM-style trajectory with proper altitude
fn ballistic_arc_position(start: DVec3, end: DVec3, t: f64, distance: f64) -> DVec3 {
    let earth_radius = 6_371_000.0;

    // Interpolate along the great circle path (surface of sphere)
    let start_normalized = start.normalize();
    let end_normalized = end.normalize();

    // Calculate the angle between start and end
    let dot = start_normalized.dot(end_normalized).clamp(-1.0, 1.0);
    let angle = dot.acos();

    // Slerp (spherical linear interpolation) for the base path
    let base_position = if angle.abs() < 0.001 {
        // Points are too close, just lerp
        start.lerp(end, t)
    } else {
        let sin_angle = angle.sin();
        let a = ((1.0 - t) * angle).sin() / sin_angle;
        let b = (t * angle).sin() / sin_angle;
        (start_normalized * a + end_normalized * b) * earth_radius
    };

    // Calculate altitude for ballistic trajectory
    // Altitude is highest at the midpoint and follows a parabolic arc
    // The maximum altitude depends on the distance traveled
    //
    // For ICBM trajectories:
    // - Short range (< 1000km): ~200km altitude
    // - Medium range (1000-5500km): ~400-800km altitude
    // - ICBM range (> 5500km): ~1200km altitude
    let max_altitude = calculate_max_altitude(distance);

    // Parabolic arc: altitude = 4 * max_altitude * t * (1 - t)
    // This gives 0 altitude at t=0 and t=1, and max_altitude at t=0.5
    let altitude = 4.0 * max_altitude * t * (1.0 - t);

    // Add altitude along the radial direction (away from Earth's center)
    let radial_direction = base_position.normalize();
    base_position + radial_direction * altitude
}

/// Calculate maximum altitude for a ballistic trajectory based on range
/// Models realistic ICBM altitude profiles
fn calculate_max_altitude(distance: f64) -> f64 {
    let distance_km = distance / 1000.0;

    // Realistic altitude scaling for different missile ranges
    if distance_km < 1000.0 {
        // Short range ballistic missiles
        150_000.0 + distance_km * 50.0
    } else if distance_km < 3500.0 {
        // Medium range ballistic missiles
        200_000.0 + (distance_km - 1000.0) * 200.0
    } else if distance_km < 5500.0 {
        // Intermediate range ballistic missiles
        700_000.0 + (distance_km - 3500.0) * 150.0
    } else {
        // Intercontinental ballistic missiles (ICBM)
        // Maximum altitude around 1200km for longest ranges
        1_000_000.0 + (distance_km - 5500.0) * 30.0
    }
}
