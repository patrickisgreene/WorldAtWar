use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_hanabi::ParticleEffect;
use bevy_log::warn;
use big_space::prelude::*;
use waw_earth::EarthOriginGrid;

use crate::{
    BallisticTo, Detonation, Maneuver, Operation, OperationIndex, OrbitAround, StopBehavior,
    TravelStraightTo, WeaponCount, WeaponHandle,
};

pub fn maneuver_index_updated(
    mut commands: Commands,
    assets: Res<AssetServer>,
    grid: Query<(Entity, &Grid), With<EarthOriginGrid>>,
    query: Query<(Entity, &Operation, &OperationIndex), Changed<OperationIndex>>,
) {
    for (entity, op, index) in query.iter() {
        if **index >= op.maneuvers.len() {
            // We Have Reached The End.
            commands.entity(entity).despawn();
            continue;
        }
        match &op.maneuvers[**index] {
            Maneuver::Stop(StopBehavior::Stop) => {}
            Maneuver::Stop(StopBehavior::Orbit {
                center,
                radius,
                length,
            }) => {
                let comp = OrbitAround {
                    center: *center,
                    radius: *radius,
                    length: *length,
                };
                commands.entity(entity).insert(comp);
            }
            Maneuver::Release {
                operation,
                formation,
                count,
                weapon,
            } => {
                let Ok((grid_entity, grid)) = grid.single() else {
                    warn!("No Grid Present!");
                    return;
                };
                commands.grid(grid_entity, grid.clone()).spawn_spatial((
                    operation.clone(),
                    WeaponCount(*count),
                    *formation,
                    WeaponHandle(weapon.clone()),
                ));

                commands.entity(entity).insert(OperationIndex(index.0 + 1));
            }
            Maneuver::Detonate => {
                let comp = Detonation::new(&assets);
                commands
                    .entity(entity)
                    .insert(ParticleEffect::new(comp.effect.clone()))
                    .insert(comp)
                    .remove::<Operation>()
                    .remove::<OperationIndex>();
            }
            Maneuver::BallisticTo(coord) => {
                let comp = BallisticTo(*coord);
                commands.entity(entity).insert(comp);
            }
            Maneuver::StraightTo(coord) => {
                let comp = TravelStraightTo(*coord);
                commands.entity(entity).insert(comp);
            }
        }
    }
}
