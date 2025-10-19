use bevy_app::{PluginGroupBuilder, prelude::*};
use bevy_internal::prelude::*;

use crate::{CliArgs, WawDefaultPlugin};

pub struct WawCorePlugins {
    pub log_level: bevy_log::Level,
    pub log_filter: String,
    pub window_title: String,
}

impl WawCorePlugins {
    pub fn from_args() -> WawCorePlugins {
        use clap::Parser;

        let matches = CliArgs::parse();

        WawCorePlugins {
            log_level: matches.log_level,
            log_filter: matches.log_filter,
            ..Default::default()
        }
    }
}

impl Default for WawCorePlugins {
    fn default() -> Self {
        Self {
            log_level: bevy_log::Level::INFO,
            log_filter: Default::default(),
            window_title: "World At War".into(),
        }
    }
}

impl PluginGroup for WawCorePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(WawDefaultPlugin::from(&self))
    }
}