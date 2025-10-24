use bevy_asset::prelude::*;
use bevy_color::prelude::*;
use bevy_ecs::prelude::*;
use bevy_image::{ImageLoaderSettings, ImageSamplerDescriptor, prelude::*};
use bevy_pbr::{ExtendedMaterial, MaterialExtension, prelude::*};
use bevy_reflect::prelude::*;
use bevy_render::render_resource::AsBindGroup;
use bevy_shader::ShaderRef;

use crate::EarthResolution;

pub const SHADER_ASSET_PATH: &str = "shaders/earth/fragment.wgsl";

pub const WATER_NORMAL_ASSET_PATH: &str = "textures/earth/water-normal.png";

pub type EarthMaterial = ExtendedMaterial<StandardMaterial, EarthExtension>;

#[derive(Resource)]
pub struct EarthMaterialHandle(Handle<EarthMaterial>);

impl EarthMaterialHandle {
    pub fn get(&self) -> Handle<EarthMaterial> {
        self.0.clone()
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct EarthExtension {
    #[texture(100)]
    #[sampler(101)]
    pub ocean_mask: Handle<Image>,
    #[texture(102)]
    #[sampler(103)]
    pub distance_map: Handle<Image>,
    #[texture(104)]
    #[sampler(105)]
    pub water_normal: Handle<Image>,
    #[texture(106)]
    #[sampler(107)]
    pub depth_map: Handle<Image>,
    #[texture(108)]
    #[sampler(109)]
    pub lake_map: Handle<Image>,
    #[texture(110)]
    #[sampler(111)]
    pub chloro_map: Handle<Image>,
    #[texture(112)]
    #[sampler(113)]
    pub road_map: Handle<Image>,
    #[texture(114)]
    #[sampler(115)]
    pub night_base_color: Handle<Image>,
    #[uniform(116)]
    pub shallow_water_color: LinearRgba,
    #[uniform(116)]
    pub medium_water_color: LinearRgba,
    #[uniform(116)]
    pub deep_water_color: LinearRgba,
    #[uniform(116)]
    pub algea_color: LinearRgba,
    #[uniform(116)]
    pub ripple_speed: f32,
    #[uniform(116)]
    pub ripple_frequency: f32,
    #[uniform(116)]
    pub ripple_distance: f32,
}

impl MaterialExtension for EarthExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

impl EarthMaterialHandle {
    pub fn create(
        assets: &AssetServer,
        materials: &mut Assets<EarthMaterial>,
        resolution: EarthResolution,
    ) -> EarthMaterialHandle {
        let water_normal = assets.load_with_settings(
            WATER_NORMAL_ASSET_PATH,
            |settings: &mut ImageLoaderSettings| {
                settings.sampler = bevy_image::ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: bevy_image::ImageAddressMode::Repeat,
                    address_mode_v: bevy_image::ImageAddressMode::Repeat,
                    ..Default::default()
                });
            },
        );

        let base_color = assets.load(format!("textures/earth/{}/base_color.png", resolution));
        let normal_map = assets.load(format!("textures/earth/{}/normal_map.png", resolution));
        let depth_map = assets.load(format!("textures/earth/{}/bathyometry.png", resolution));
        let ocean_mask = assets.load(format!("textures/earth/{}/ocean_mask.png", resolution));
        let distance_map = assets.load(format!("textures/earth/{}/distance_map.png", resolution));
        let road_map = assets.load(format!("textures/earth/{}/road_mask.png", resolution));
        let lake_map = assets.load(format!("textures/earth/{}/lake_mask.png", resolution));
        let night_base_color = assets.load(format!(
            "textures/earth/{}/night_base_color.png",
            resolution
        ));
        let chloro_map = assets.load(format!("textures/earth/{}/chlorophyll.png", resolution));

        EarthMaterialHandle(materials.add(EarthMaterial {
            base: StandardMaterial {
                base_color_texture: Some(base_color),
                normal_map_texture: Some(normal_map),
                ..Default::default()
            },
            extension: EarthExtension {
                water_normal: water_normal,
                depth_map,
                ocean_mask,
                distance_map,
                lake_map,
                chloro_map,
                road_map,
                night_base_color,
                shallow_water_color: LinearRgba::new(0.04, 0.35, 0.55, 1.0),
                medium_water_color: LinearRgba::new(0.02, 0.25, 0.45, 1.0),
                deep_water_color: LinearRgba::new(0.0, 0.05, 0.15, 1.0),
                algea_color: LinearRgba::new(0.15, 0.35, 0.20, 1.0),
                ripple_speed: 5.0,
                ripple_frequency: 6000.0,
                ripple_distance: 0.001,
            },
        }))
    }
}
