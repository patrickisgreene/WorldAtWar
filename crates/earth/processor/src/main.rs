use clap::Parser;

mod args;
mod processing;
mod textures;

pub use args::{CliArgs, EarthCommand};

fn main() {
    let args = CliArgs::parse();

    match &args.command {
        &EarthCommand::Topography { .. } => textures::handle_topography(&args),
        &EarthCommand::Bathyometry { .. } => textures::handle_bathyometry(&args),
        &EarthCommand::BaseColor { .. } => textures::handle_base_color(&args),
        &EarthCommand::NormalMap { .. } => textures::handle_normal_map(&args),
        &EarthCommand::OceanMask { .. } => textures::handle_ocean_mask(&args),
        &EarthCommand::LakeMask { .. } => textures::handle_lake_mask(&args),
        &EarthCommand::RiverMask { .. } => textures::handle_river_mask(&args),
        &EarthCommand::RoadMask { .. } => textures::handle_road_mask(&args),
        &EarthCommand::LightMask { .. } => textures::handle_light_mask(&args),
        &EarthCommand::Chlorophyll { .. } => textures::handle_chlorophyll(&args),
        &EarthCommand::UrbanAreasMask { .. } => textures::handle_urban_areas_mask(&args),
        &EarthCommand::DistanceMap { .. } => textures::handle_distance_map(&args),
        &EarthCommand::NightBaseColor { .. } => textures::handle_night_base_color(&args),
        &EarthCommand::Initialize => {
            macro_rules! run_subcommand {
                ($arg:tt, $default:tt, $func:tt) => {
                    let command = EarthCommand::$arg {
                        input: args::command::$default(),
                        output: args::command::earth_output_default(),
                        resolutions: args::default_resolutions(),
                    };
                    let args = CliArgs { command };
                    crate::textures::$func(&args);
                };
            }
            run_subcommand!(Topography, topography_input_default, handle_topography);
            run_subcommand!(Bathyometry, bathyometry_input_default, handle_bathyometry);
            run_subcommand!(BaseColor, base_color_input_default, handle_base_color);
            run_subcommand!(NormalMap, topography_input_default, handle_normal_map);
            run_subcommand!(OceanMask, ocean_mask_input_default, handle_ocean_mask);
            run_subcommand!(RiverMask, river_mask_input_default, handle_river_mask);
            run_subcommand!(LakeMask, lake_mask_input_default, handle_lake_mask);
            run_subcommand!(DistanceMap, ocean_mask_input_default, handle_distance_map);
            run_subcommand!(RoadMask, road_mask_input_default, handle_road_mask);
            run_subcommand!(Chlorophyll, chlorophyll_input_default, handle_chlorophyll);
            run_subcommand!(LightMask, light_mask_input_default, handle_light_mask);
            run_subcommand!(
                UrbanAreasMask,
                urban_areas_input_default,
                handle_urban_areas_mask
            );
            run_subcommand!(
                NightBaseColor,
                night_base_color_input_default,
                handle_night_base_color
            );
        }
    }
}
