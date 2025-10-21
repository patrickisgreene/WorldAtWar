use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_internal::scene::SceneRoot;
use bevy_log::warn;
use bevy_math::prelude::*;
use bevy_transform::prelude::*;
use big_space::prelude::*;
use waw_earth::EarthOriginGrid;
use waw_weapons::data::Weapon;

use crate::{Formation, Operation, WeaponCount, WeaponHandle};

const ECHELON_ANGLE: f32 = 45.0_f32.to_radians();

pub fn initialize_weapon_instances(
    grid: Query<&Grid, With<EarthOriginGrid>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    weapons: Res<Assets<Weapon>>,
    mut query: Query<
        (Entity, &Operation, &Formation, &WeaponCount, &WeaponHandle),
        Without<Children>,
    >,
) {
    for (entity, op, formation, count, handle) in query.iter_mut() {
        if let Some(weapon) = weapons.get(&**handle) {
            let Ok(grid) = grid.single() else {
                warn!("No Grid Present!");
                continue;
            };

            let (cell, trans) = grid.translation_to_grid(op.starting.world_pos());

            // Rotate the parent entity to align with Earth's surface
            let rotation = op.starting.rotation();

            // Set the correct position for the intitial starting point.
            commands.entity(entity)
                .insert((
                    cell,
                    Transform::from_translation(trans).with_rotation(rotation),
                ));

            let positions =
                formation_to_positions(formation, &count, weapon.model.transformation.scale * 7.0);

            for i in 0..**count {
                commands.entity(entity).with_child((
                    Name::new(format!("Weapon #{i}")),
                    SceneRoot(assets.load(weapon.scene())),
                    weapon.model.transformation.transform(positions[i]),
                ));
            }
        }
    }
}

fn formation_to_positions(formation: &Formation, &count: &usize, spacing: f32) -> Vec<Vec3> {
    let mut output = vec![];

    match formation {
        Formation::Chevron => {
            let half_count = (count + 1) / 2;

            // Right side of the V
            for i in 0..half_count {
                let offset_distance = i as f32 * spacing;
                let x_offset = offset_distance * ECHELON_ANGLE.cos();
                let z_offset = offset_distance * ECHELON_ANGLE.sin();
                output.push(Vec3::new(x_offset, 0.0, -z_offset));
            }

            // Left side of the V (mirror across the center)
            for i in 1..=(count - half_count) {
                let offset_distance = i as f32 * spacing;
                let x_offset = -offset_distance * ECHELON_ANGLE.cos();
                let z_offset = offset_distance * ECHELON_ANGLE.sin();
                output.push(Vec3::new(x_offset, 0.0, -z_offset));
            }
        }
        Formation::DiagonalLeft => {
            for i in 0..count {
                let offset_distance = i as f32 * spacing;
                let x_offset = offset_distance * ECHELON_ANGLE.cos();
                let z_offset = offset_distance * ECHELON_ANGLE.sin();
                output.push(Vec3::new(x_offset, 0.0, -z_offset));
            }
        }
        Formation::DiagonalRight => {
            for i in 0..count {
                let offset_distance = i as f32 * spacing;
                let x_offset = offset_distance * ECHELON_ANGLE.cos();
                let z_offset = offset_distance * ECHELON_ANGLE.sin();
                output.push(Vec3::new(-x_offset, 0.0, -z_offset));
            }
        }
    }

    output
}
