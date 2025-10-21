use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_image::prelude::*;

use crate::EarthResolution;

#[derive(Resource, PartialEq)]
pub struct EarthHeightMap(pub(crate) Handle<Image>);

impl EarthHeightMap {
    pub fn create(assets: &AssetServer, resolution: EarthResolution) -> EarthHeightMap {
        EarthHeightMap(assets.load(format!("textures/earth/{}/topography.png", resolution)))
    }
    pub fn get(&self) -> Handle<Image> {
        self.0.clone()
    }
}

#[derive(Resource, PartialEq)]
pub struct EarthUrbanAreas(pub(crate) Handle<Image>);

impl EarthUrbanAreas {
    pub fn create(assets: &AssetServer, resolution: EarthResolution) -> EarthUrbanAreas {
        EarthUrbanAreas(assets.load(format!("textures/earth/{}/urban_areas.png", resolution)))
    }
    pub fn get(&self) -> Handle<Image> {
        self.0.clone()
    }
}
