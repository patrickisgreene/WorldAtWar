use bevy_ecs::prelude::*;

#[derive(Component)]
pub enum RadarShape {
    Cone { radius: f64, length: f64 },
    Sphere { radius: f64 },
}