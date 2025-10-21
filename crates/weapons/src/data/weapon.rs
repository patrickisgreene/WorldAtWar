use std::path::PathBuf;

use bevy_gltf::prelude::*;
use bevy_reflect::TypePath;
use super::{Movement, Transformation};
use serde::{Deserialize, Serialize};
use bevy_asset::{AssetPath, prelude::*};

#[derive(Serialize, Deserialize)]
pub struct Model {
    pub file: PathBuf,
    #[serde(default)]
    pub transformation: Transformation,
}

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub movement: Movement,
    pub model: Model,
}

impl Weapon {
    pub fn asset_path(&self) -> String {
        format!(
            "models/{}",
            self.model.file.to_string_lossy()
        )
    }

    pub fn scene(&self) -> AssetPath<'static> {
        GltfAssetLabel::Scene(0).from_asset(self.asset_path())
    }
}
