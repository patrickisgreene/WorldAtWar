use bevy::math::{DQuat, DVec3};
use bevy::{
    core_pipeline::tonemapping::Tonemapping, input::common_conditions::input_toggle_active,
    prelude::*,
};
use big_space::camera::BigSpaceCameraInput;
use big_space::prelude::*;

use waw_operation::{Formation, Maneuver, Operation, OperationsPlugin, OrbitLength, WeaponCount, WeaponHandle};
use waw_core::WawCorePlugins;
use waw_earth::{EarthLevelOfDetailFocus, EarthOriginGrid, EarthPlugin, EarthResolution};
use waw_geocoord::GeoCoord;
use waw_utils::consts::{EARTH_RADIUS, GEOSYNCHRONUS_ORBIT};
use waw_weapons::WeaponsPlugin;

fn main() {
    App::new()
        .add_plugins((
            WawCorePlugins::from_args(),
            EarthPlugin,
            WeaponsPlugin,
            waw_radar::RadarPlugin,
            OperationsPlugin,
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
        .add_systems(Startup, spawn_operation)
        .add_systems(
            Update,
            escape_key_to_camera
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

pub fn escape_key_to_camera(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<BigSpaceCameraInput>) {
    if keys.just_pressed(KeyCode::Escape) {
        input.defaults_disabled = !input.defaults_disabled;
    }
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

fn spawn_operation(mut commands: Commands, assets: Res<AssetServer>, grid: Query<(Entity, &Grid), With<EarthOriginGrid>>) {
    let Ok((grid_entity, grid)) = grid.single() else {
        warn!("No Grid Present!");
        return;
    };
    commands.grid(grid_entity, grid.clone())
        .spawn_spatial((
            Operation {
                starting: GeoCoord::DALLAS,
                maneuvers: vec![
                    Maneuver::StraightTo(GeoCoord::DALLAS),
                    Maneuver::Release {
                        operation: Operation {
                            starting: GeoCoord::DALLAS,
                            maneuvers: vec![
                                Maneuver::BallisticTo(GeoCoord::WASHINGTON_DC),
                                Maneuver::Detonate
                            ]
                        },
                        formation: Formation::Chevron,
                        count: 1,
                        weapon: assets.load("weapons/aim7.weapon.ron")
                    },
                    Maneuver::StraightTo(GeoCoord::MIAMI),
                ]
            },
            WeaponCount(5),
            Formation::Chevron,
            WeaponHandle(assets.load("weapons/b2.weapon.ron"))
        ));

    commands.grid(grid_entity, grid.clone())
        .spawn_spatial((
            Operation {
                starting: GeoCoord::DALLAS,
                maneuvers: vec![
                    Maneuver::Stop(waw_operation::StopBehavior::Orbit { center: GeoCoord::MIAMI, radius: 10_000.0, length: OrbitLength::Indefinite })
                ]
            },
            WeaponCount(5),
            Formation::Chevron,
            WeaponHandle(assets.load("weapons/b2.weapon.ron"))
        )).id();

    commands.grid(grid_entity, grid.clone())
        .spawn_spatial((
            Operation {
                starting: GeoCoord::HOUSTON,
                maneuvers: vec![
                    Maneuver::BallisticTo(GeoCoord::MIAMI),
                    Maneuver::Detonate
                ]
            },
            WeaponCount(5),
            Formation::Chevron,
            WeaponHandle(assets.load("weapons/aim7.weapon.ron"))
        ));
    
}
