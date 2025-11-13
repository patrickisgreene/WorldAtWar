use crate::{
    earth::EarthConfig,
    data::{TileAtlas, TileTree},
    plugin::EarthSettings,
    view::{EarthViewComponents, EarthViewConfig},
};
use bevy::{ecs::system::SystemState, prelude::*, render::storage::ShaderStorageBuffer};
use big_space::floating_origins::BigSpace;

#[derive(Clone)]
pub(crate) struct EarthToSpawn<M: Material + Clone> {
    config: Handle<EarthConfig>,
    view_config: EarthViewConfig,
    material: M,
    view: Entity,
}

#[derive(Resource)]
pub(crate) struct EarthsToSpawn<M: Material>(pub(crate) Vec<EarthToSpawn<M>>);

pub(crate) fn spawn_earths<M: Material>(
    mut commands: Commands,
    mut earths: ResMut<EarthsToSpawn<M>>,
    asset_server: Res<AssetServer>,
) {
    earths.0.retain(|earth| {
        if asset_server.is_loaded(&earth.config) {
            let earth = earth.clone();

            commands.queue(move |world: &mut World| {
                let EarthToSpawn {
                    config,
                    view_config,
                    material,
                    view,
                } = earth;

                let mut state = SystemState::<(
                    Commands,
                    Res<Assets<EarthConfig>>,
                    Query<Entity, With<BigSpace>>,
                    ResMut<Assets<M>>,
                    ResMut<EarthViewComponents<TileTree>>,
                    ResMut<Assets<ShaderStorageBuffer>>,
                    Res<EarthSettings>,
                )>::new(world);

                let (
                    mut commands,
                    configs,
                    big_space,
                    mut materials,
                    mut tile_trees,
                    mut buffers,
                    settings,
                ) = state.get_mut(world);

                let config = configs.get(config.id()).unwrap().clone();

                let root = big_space.single().unwrap();

                let earth = commands
                    .spawn((
                        config.shape.transform(),
                        TileAtlas::new(&config, &mut buffers, &settings),
                        MeshMaterial3d(materials.add(material)),
                    ))
                    .id();

                commands.entity(root).add_child(earth);

                tile_trees.insert(
                    (earth, view),
                    TileTree::new(
                        &config,
                        &view_config,
                        (earth, view),
                        &mut commands,
                        &mut buffers,
                    ),
                );

                state.apply(world);
            });
            false
        } else {
            true
        }
    });
}

pub trait SpawnEarthCommandsExt<M: Material> {
    // define a method that we will be able to call on `commands`
    fn spawn_earth(
        &mut self,
        config: Handle<EarthConfig>,
        view_config: EarthViewConfig,
        material: M,
        view: Entity,
    );
}

impl<M: Material> SpawnEarthCommandsExt<M> for Commands<'_, '_> {
    fn spawn_earth(
        &mut self,
        config: Handle<EarthConfig>,
        view_config: EarthViewConfig,
        material: M,
        view: Entity,
    ) {
        self.queue(move |world: &mut World| {
            world
                .resource_mut::<EarthsToSpawn<M>>()
                .0
                .push(EarthToSpawn {
                    config,
                    view_config,
                    material,
                    view,
                });
        });
    }
}
