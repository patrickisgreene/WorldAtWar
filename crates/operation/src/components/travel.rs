use bevy_ecs::prelude::*;
use waw_geocoord::GeoCoord;

#[derive(Component)]
#[require(TravelStraightToState)]
pub struct TravelStraightTo(pub GeoCoord);

#[derive(Default, Component)]
pub struct TravelStraightToState {
    pub progress: f64,
    pub start: Option<GeoCoord>
}