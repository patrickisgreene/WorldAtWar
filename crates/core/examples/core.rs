use bevy::prelude::*;

use waw_core::WawCorePlugins;

fn main() {
    App::new().add_plugins(WawCorePlugins::from_args()).run();
}
