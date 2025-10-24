use bevy_ecs::prelude::*;
use bevy_time::prelude::*;

use crate::Detonation;

pub fn cleanup_detonation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Detonation)>,
) {
    for (entity, mut detonation) in query.iter_mut() {
        detonation.timer.tick(time.delta());
        // Despawn the entity once the timer has finished
        // This ensures all particles have expired (lifetime is 2 seconds)
        if detonation.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
