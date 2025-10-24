use bevy_ecs::prelude::*;
use bevy_internal::prelude::*;
use waw_geocoord::GeoCoord;
use waw_weapons::data::Weapon;

use crate::{Formation, OrbitLength};

#[derive(Component, Default, Debug, PartialEq, Clone, Copy, Deref)]
pub struct OperationIndex(pub usize);

#[derive(Component, Debug, PartialEq, Clone)]
#[require(OperationIndex)]
pub struct Operation {
    pub starting: GeoCoord,
    pub maneuvers: Vec<Maneuver>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Maneuver {
    Detonate,
    Release {
        operation: Operation,
        formation: Formation,
        count: usize,
        weapon: Handle<Weapon>,
    },
    StraightTo(GeoCoord),
    BallisticTo(GeoCoord),
    Stop(StopBehavior),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StopBehavior {
    Stop,
    Orbit {
        center: GeoCoord,
        radius: f64,
        length: OrbitLength,
    },
}
