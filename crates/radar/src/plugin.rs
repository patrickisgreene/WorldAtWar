use bevy_app::prelude::*;

use crate::gizmos::draw_radar_gizmos;

pub struct RadarPlugin;

impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_radar_gizmos);
    }
}