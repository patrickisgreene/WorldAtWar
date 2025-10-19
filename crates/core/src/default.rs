use bevy_app::{PluginGroupBuilder, prelude::*};
use bevy_asset::prelude::*;
use bevy_internal::prelude::*;
use bevy_log::LogPlugin;

use crate::{CliArgs, WawCorePlugins};

pub struct WawDefaultPlugin {
    pub log_level: bevy_log::Level,
    pub log_filter: String,
    pub window_title: String,
}

impl Default for WawDefaultPlugin {
    fn default() -> Self {
        Self {
            log_level: bevy_log::Level::INFO,
            log_filter: bevy_log::DEFAULT_FILTER.into(),
            window_title: "World At War".into(),
        }
    }
}

impl WawDefaultPlugin {
    pub fn asset_plugin(&self) -> AssetPlugin {
        let file_path = format!("{}/assets", env!("CARGO_MANIFEST_PATH"));
        AssetPlugin {
            file_path,
            ..Default::default()
        }
    }

    pub fn window_plugin(&self) -> WindowPlugin {
        WindowPlugin {
            primary_window: Some(Window {
                title: self.window_title.clone(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn log_plugin(&self) -> LogPlugin {
        LogPlugin {
            filter: self.log_filter.clone(),
            level: self.log_level.clone(),
            ..Default::default()
        }
    }

    pub fn default_plugins(&self) -> PluginGroupBuilder {
        DefaultPlugins
            .set(self.log_plugin())
            .set(self.asset_plugin())
            .set(self.window_plugin())
    }

    pub fn from_args() -> WawDefaultPlugin {
        use clap::Parser;

        let matches = CliArgs::parse();

        WawDefaultPlugin {
            log_level: matches.log_level,
            log_filter: matches.log_filter,
            ..Default::default()
        }
    }
}

impl Plugin for WawDefaultPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(self.default_plugins());
    }
}

impl <'a>From<&'a WawCorePlugins> for WawDefaultPlugin {
    fn from(value: &'a WawCorePlugins) -> Self {
        WawDefaultPlugin {
            log_level: value.log_level,
            log_filter: value.log_filter.clone(),
            window_title: value.window_title.clone()
        }
    }
}