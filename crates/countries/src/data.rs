use bevy_reflect::TypePath;
use waw_geocoord::GeoCoord;
use serde::{Serialize, Deserialize};
use bevy_asset::prelude::*;

#[derive(Asset, TypePath, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Country {
    pub name: String,
    pub code: String,
    pub population: u64,
    pub capital: usize,
    pub cities: Vec<City>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub coordinates: Coordinates
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64
}

impl <'a>From<&'a Coordinates> for GeoCoord {
    fn from(value: &'a Coordinates) -> Self {
        GeoCoord::new(bevy_math::DVec2::new(value.lat, value.lon))
    }
}