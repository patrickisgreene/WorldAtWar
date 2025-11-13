pub mod debug;
pub mod earth;
pub mod data;
pub mod math;
pub mod picking;
pub mod plugin;
pub mod preprocess;
pub mod render;
pub mod shaders;
pub mod utils;
pub mod view;
pub mod material;

#[doc(hidden)]
pub mod prelude {
    pub use crate::{
        debug::{
            DebugCameraController, DebugEarthMaterial, EarthDebugPlugin, LoadingImages,
            OrbitalCameraController,
        },
        earth::EarthConfig,
        data::{
            AttachmentConfig, AttachmentFormat, AttachmentLabel, GpuTileAtlas, TileAtlas, TileTree,
        },
        material::EarthMaterial,
        math::{EarthShape, TileCoordinate},
        picking::{EarthPickingPlugin, PickingData},
        plugin::{EarthPlugin, EarthSettings},
        render::EarthMaterialPlugin,
        utils::SpawnEarthCommandsExt,
        view::{EarthViewComponents, EarthViewConfig},
    };
    pub use big_space::{commands::BigSpaceCommands, grid::Grid};
}
