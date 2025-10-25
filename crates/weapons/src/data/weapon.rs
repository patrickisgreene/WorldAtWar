use super::{Movement, Radar, Model};
use bevy_asset::{AssetPath, prelude::*};
use bevy_gltf::prelude::*;
use bevy_reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub movement: Movement,
    pub model: Model,
    pub radar: Option<Radar>,
}

impl Weapon {
    pub fn asset_path(&self) -> String {
        format!("models/{}", self.model.file.to_string_lossy())
    }

    pub fn scene(&self) -> AssetPath<'static> {
        GltfAssetLabel::Scene(0).from_asset(self.asset_path())
    }
}
