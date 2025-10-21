use bevy_ecs::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub enum Formation {
    Chevron,
    DiagonalRight,
    DiagonalLeft,
}