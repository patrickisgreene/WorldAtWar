use std::path::PathBuf;

use super::{Movement, Transformation};
use bevy_asset::{AssetPath, prelude::*};
use bevy_gltf::prelude::*;
use bevy_reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Radar {
    pub strength: f32,
    pub shape: RadarShape,
    #[serde(default)]
    pub transformation: Transformation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RadarShape {
    Sphere(f64),
    Cone { radius: f64, length: f64 },
}

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
