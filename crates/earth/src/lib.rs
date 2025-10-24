#![feature(iter_array_chunks)]

mod detail;
mod plugin;

pub mod geometry;
pub mod material;
pub mod systems;

pub use material::EarthMaterialHandle;
pub use plugin::EarthPlugin;

use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct EarthLevelOfDetailFocus;

#[derive(Component, Default)]
pub struct EarthOriginGrid;

use std::fmt;

#[derive(Resource, Default, Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum EarthResolution {
    Low,
    Mid,
    #[default]
    High,
    Ultra,
    Max,
}

impl fmt::Display for EarthResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EarthResolution::Low => write!(f, "2700x1350"),
            EarthResolution::Mid => write!(f, "5400x2700"),
            EarthResolution::High => write!(f, "8100x4050"),
            EarthResolution::Ultra => write!(f, "10800x5400"),
            EarthResolution::Max => write!(f, "21600x10800"),
        }
    }
}
