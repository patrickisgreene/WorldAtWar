use bevy_ecs::prelude::*;
use waw_geocoord::GeoCoord;

#[derive(Component)]
#[require(BallisticToState)]
pub struct BallisticTo(pub GeoCoord);

#[derive(Default, Component)]
pub struct BallisticToState {
    pub progress: f64,
    pub start: Option<GeoCoord>,
}
