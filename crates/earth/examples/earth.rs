use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use waw_earth::material::EarthMaterial;
use waw_earth::prelude::*;

pub const EARTH_RADIUS: f64 = 6_371_000.0; // meters

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "../../assets".into(),
                    ..default()
                })
                .build()
                .disable::<TransformPlugin>(),
            FpsOverlayPlugin::default(),
            EarthPlugin,
            EarthDebugPlugin, // enable debug settings and controls
            EarthPickingPlugin,
        ))
        .insert_resource(EarthSettings::default())
        // .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, (initialize, spawn_lighting).chain())
        .run();
}

pub fn spawn_lighting(mut commands: Commands, grid: Query<(Entity, &Grid)>) {
    let Ok((grid_entity, grid)) = grid.single() else {
        warn!("Lighting system called before the root grid exists");
        return;
    };

    let (grid_cell, grid_trans) =
        grid.translation_to_grid(DVec3::NEG_Z * (2.0 * EARTH_RADIUS));
    let direction = (-DVec3::NEG_X * (2.0 * EARTH_RADIUS)).normalize();
    let rotation = DQuat::from_rotation_arc(-DVec3::NEG_X, direction);
    commands.grid(grid_entity, grid.clone()).spawn_spatial((
        grid_cell,
        DirectionalLight {
            illuminance: 0.,
            ..default()
        },
        Transform::from_translation(grid_trans).with_rotation(rotation.as_quat()),
    ));
}

fn initialize(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut view = Entity::PLACEHOLDER;

    commands.spawn_big_space(Grid::default(), |root| {
        view = root
            .spawn_spatial((
                Transform::from_translation(-Vec3::X * EARTH_RADIUS as f32 * 3.0)
                    .looking_to(Vec3::X, Vec3::Y),
                DebugCameraController::new(EARTH_RADIUS),
                OrbitalCameraController::default(),
            ))
            .id();
    });

    commands.spawn_earth(
        asset_server.load("earth/data/config.tc.ron"),
        EarthViewConfig::default(),
        EarthMaterial {
                water_normal: asset_server.load_with_settings(
                "earth/water-normal.png",
                |s: &mut _| {
                    *s = ImageLoaderSettings {
                        sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                            // rewriting mode to repeat image,
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            ..default()
                        }),
                        ..default()
                    }
                },
            ),
            shallow_water_color: LinearRgba::new(0.04, 0.35, 0.55, 1.0),
            medium_water_color: LinearRgba::new(0.02, 0.25, 0.45, 1.0),
            deep_water_color: LinearRgba::new(0.0, 0.05, 0.15, 1.0),
            ripple_speed: 5.0,
            ripple_frequency: 6000.0,
            ripple_distance: 0.001,
        },
        view,
    );
}
