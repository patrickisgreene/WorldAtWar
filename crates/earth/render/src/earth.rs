use crate::{
    data::{AttachmentConfig, AttachmentLabel},
    math::{EarthShape, TileCoordinate},
};
use bevy::{
    asset::ron, ecs::entity::hash_map::EntityHashMap, platform::collections::HashMap, prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Deref, DerefMut, Resource)]
pub struct EarthComponents<C>(EntityHashMap<C>);

impl<C> Default for EarthComponents<C> {
    fn default() -> Self {
        Self(default())
    }
}

#[derive(Serialize, Deserialize, Asset, TypePath, Debug, Clone)]
pub struct EarthConfig {
    pub path: String,
    pub shape: EarthShape,
    pub lod_count: u32,
    pub min_height: f32,
    pub max_height: f32,
    pub attachments: HashMap<AttachmentLabel, AttachmentConfig>,
    pub tiles: Vec<TileCoordinate>,
}

impl Default for EarthConfig {
    fn default() -> Self {
        Self {
            shape: EarthShape::WGS84,
            lod_count: 6,
            min_height: 0.0,
            max_height: 1.0,
            path: default(),
            tiles: default(),
            attachments: default(),
        }
    }
}

impl EarthConfig {
    pub fn add_attachment(
        &mut self,
        label: AttachmentLabel,
        attachment: AttachmentConfig,
    ) -> &mut Self {
        self.attachments.insert(label, attachment);
        self
    }

    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let encoded = fs::read_to_string(path)?;
        Ok(ron::from_str(&encoded)?)
    }

    pub fn save_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let encoded = ron::ser::to_string_pretty(self, default())?;
        Ok(fs::write(path, encoded)?)
    }
}
