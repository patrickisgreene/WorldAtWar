use bevy_app::prelude::*;
use waw_ron_asset::RonAssetPlugin;

use crate::data::Weapon;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<Weapon>::new("weapon.ron"));
    }
}
