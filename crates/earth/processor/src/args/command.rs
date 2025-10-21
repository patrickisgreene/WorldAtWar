use clap::Subcommand;
use std::path::PathBuf;

use super::{default_resolutions, ImageResolution, ImageResolutionParser};

#[derive(Subcommand)]
pub enum EarthCommand {
    Initialize,
    Topography {
        #[arg(short, long, default_value_os_t = topography_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    Bathyometry {
        #[arg(short, long, default_value_os_t = bathyometry_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    BaseColor {
        #[arg(short, long, default_value_os_t = base_color_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    NightBaseColor {
        #[arg(short, long, default_value_os_t = night_base_color_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    NormalMap {
        #[arg(short, long, default_value_os_t = topography_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    DistanceMap {
        #[arg(short, long, default_value_os_t = distance_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    OceanMask {
        #[arg(short, long, default_value_os_t = ocean_mask_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    LakeMask {
        #[arg(short, long, default_value_os_t = lake_mask_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    RiverMask {
        #[arg(short, long, default_value_os_t = river_mask_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    RoadMask {
        #[arg(short, long, default_value_os_t = road_mask_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    LightMask {
        #[arg(short, long, default_value_os_t = light_mask_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    UrbanAreasMask {
        #[arg(short, long, default_value_os_t = urban_areas_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
    Chlorophyll {
        #[arg(short, long, default_value_os_t = chlorophyll_input_default())]
        input: PathBuf,
        #[arg(short, long, default_value_os_t = earth_output_default())]
        output: PathBuf,
        #[arg(short, long, value_parser = ImageResolutionParser, default_values_t = default_resolutions())]
        resolutions: Vec<ImageResolution>
    },
}

pub fn distance_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.extend(["shapefiles", "ne_10m_countries", "ne_10m_admin_0_countries.shp"]);
    path
}

pub fn ocean_mask_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "shapefiles", "ne_10m_countries", "ne_10m_admin_0_countries.shp"]);
    path
}

pub fn lake_mask_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "shapefiles", "ne_10m_lakes", "ne_10m_lakes.shp"]);
    path
}

pub fn urban_areas_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "shapefiles", "ne_10m_urban_areas", "ne_10m_urban_areas.shp"]);
    path
}

pub fn river_mask_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "shapefiles", "ne_10m_rivers", "ne_10m_rivers_lake_centerlines.shp"]);
    path
}

pub fn road_mask_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "shapefiles", "ne_10m_roads", "ne_10m_roads.shp"]);
    path
}

pub fn light_mask_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "images", "black_marble_2016.jpg"]);
    path
}

pub fn chlorophyll_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "images", "chlorophyll.png"]);
    path
}

pub fn topography_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "images", "topography.png"]);
    path
}

pub fn earth_output_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["assets", "textures", "earth"]);
    path
}

pub fn bathyometry_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "images", "bathyometry.png"]);
    path
}

pub fn base_color_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "images", "february_blue_marble.jpg"]);
    path
}

pub fn night_base_color_input_default() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.extend(["data", "images", "black_marble_stitched.png"]);
    path
}