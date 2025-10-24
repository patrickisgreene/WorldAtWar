use bevy_ecs::prelude::*;
use waw_geocoord::GeoCoord;

#[derive(Component)]
#[require(OrbitAroundState)]
pub struct OrbitAround {
    pub center: GeoCoord,
    pub radius: f64,
    pub length: OrbitLength,
}

#[derive(Component, Default)]
pub struct OrbitAroundState {
    pub progress: f64,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum OrbitLength {
    Count(usize),
    Indefinite,
}

impl Default for OrbitLength {
    fn default() -> Self {
        OrbitLength::Count(1)
    }
}
