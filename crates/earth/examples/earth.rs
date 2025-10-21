use bevy::math::{DQuat, DVec3};
use bevy::{
    core_pipeline::tonemapping::Tonemapping, input::common_conditions::input_toggle_active,
    prelude::*,
};
use big_space::prelude::*;

use waw_core::WawCorePlugins;
use waw_earth::{EarthLevelOfDetailFocus, EarthOriginGrid, EarthPlugin, EarthResolution};
use waw_utils::consts::{EARTH_RADIUS, GEOSYNCHRONUS_ORBIT};

fn main() {
    App::new()
        .add_plugins((
            WawCorePlugins::from_args(),
            EarthPlugin,
            BigSpaceDefaultPlugins,
        ))
        .insert_resource(EarthResolution::Low)
        .insert_resource(AmbientLight {
            brightness: 500.0,
            ..Default::default()
        })
        .add_plugins((
            bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::new()
                .run_if(input_toggle_active(false, KeyCode::F11)),
        ))
        .add_systems(
            PreStartup,
            (spawn_grid, (spawn_lighting, spawn_camera).after(spawn_grid)),
        )
        .run();
}

pub fn spawn_grid(mut commands: Commands) {
    commands.spawn_big_space_default(|root_grid| {
        root_grid.insert(EarthOriginGrid);

        let (grid_cell, grid_offset) = root_grid.grid().translation_to_grid(DVec3::ZERO);

        root_grid.spawn_spatial((
            grid_cell,
            FloatingOrigin,
            Transform::from_translation(grid_offset),
        ));
    });
}

pub fn spawn_lighting(mut commands: Commands, grid: Query<(Entity, &Grid), With<EarthOriginGrid>>) {
    let Ok((grid_entity, grid)) = grid.single() else {
        warn!("Lighting system called before the root grid exists");
        return;
    };

    let (grid_cell, grid_trans) =
        grid.translation_to_grid(DVec3::NEG_Z * (GEOSYNCHRONUS_ORBIT + EARTH_RADIUS));
    let direction = (-DVec3::NEG_X * (GEOSYNCHRONUS_ORBIT + EARTH_RADIUS)).normalize();
    let rotation = DQuat::from_rotation_arc(-DVec3::NEG_X, direction);
    commands.grid(grid_entity, grid.clone()).spawn_spatial((
        grid_cell,
        DirectionalLight::default(),
        Transform::from_translation(grid_trans).with_rotation(rotation.as_quat()),
    ));
}

pub fn spawn_camera(mut commands: Commands, grid: Query<(Entity, &Grid), With<EarthOriginGrid>>) {
    if let Ok((entity, grid)) = grid.single() {
        let elevation = EARTH_RADIUS + GEOSYNCHRONUS_ORBIT * 0.10;
        let pos = DVec3::new(0.0, 0.0, elevation);
        let (grid_cell, grid_offset) = grid.translation_to_grid(pos);
        let target = (-pos).as_vec3();
        commands.grid(entity, grid.clone()).spawn_spatial((
            grid_cell,
            EarthLevelOfDetailFocus,
            Camera3d::default(),
            Camera {
                order: 0,
                ..Default::default()
            },
            Tonemapping::AcesFitted,
            BigSpaceCameraController::default(),
            Transform::from_translation(grid_offset).looking_at(target, Vec3::Y),
        ));
    }
}