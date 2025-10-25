use bevy_ecs::prelude::*;
use bevy_math::prelude::*;

mod shape;
mod gizmos;
mod plugin;

pub use gizmos::*;
pub use shape::RadarShape;
pub use plugin::RadarPlugin;

#[derive(Component)]
pub struct Radar {
    pub strength: f32,
    pub rotation: Quat,
    pub translation: Vec3,
}

#[derive(Component)]
pub struct RadarCrossSection(pub f32);