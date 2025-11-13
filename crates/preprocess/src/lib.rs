mod cli;
mod core;
mod process;

use crate::{
    cli::PreprocessBar,
    core::create_mask_and_fill_no_data,
    core::{PreprocessContext, clear_directory, delete_directory},
    process::{downsample_and_stitch, reproject, reproject_to_tiles, split_and_stitch},
};
use gdal::{
    Dataset,
    raster::{GdalDataType, GdalType},
};
use num::NumCast;
use std::time::Instant;
use waw_earth::prelude::*;

pub mod prelude {
    pub use crate::{
        cli::{EarthCli, Cli, Command},
        core::{PreprocessContext, PreprocessDataType, PreprocessNoData},
        preprocess,
        preprocess_streaming,
    };
}

fn preprocess_gen<T: Copy + GdalType + PartialEq + NumCast>(
    src_dataset: Dataset,
    context: &mut PreprocessContext,
) {
    if context.overwrite {
        clear_directory(&context.tile_dir);
    }

    clear_directory(&context.temp_dir);

    let start_preprocessing = Instant::now();

    let progress_bar = PreprocessBar::new("Reprojecting".to_string());
    let faces = reproject::<T>(src_dataset, context, Some(progress_bar.callback())).unwrap();
    progress_bar.finish();

    let progress_bar = PreprocessBar::new("Splitting".to_string());
    let tiles = split_and_stitch::<T>(faces, context, Some(progress_bar.callback())).unwrap();
    progress_bar.finish();

    let progress_bar = PreprocessBar::new("Downsampling".to_string());
    let tiles = downsample_and_stitch::<T>(&tiles, context, Some(progress_bar.callback())).unwrap();
    progress_bar.finish();

    let progress_bar = PreprocessBar::new("Filling".to_string());
    create_mask_and_fill_no_data(&tiles, context, Some(progress_bar.callback())).unwrap();
    progress_bar.finish();

    delete_directory(&context.temp_dir);

    save_earth_config(tiles, context);

    println!("Preprocessing took: {:?}", start_preprocessing.elapsed());
}

fn preprocess_streaming_gen<T: Copy + GdalType + PartialEq + NumCast + Send + Sync + std::fmt::Debug>(
    src_dataset: Dataset,
    context: &mut PreprocessContext,
) {
    if context.overwrite {
        clear_directory(&context.tile_dir);
    }

    clear_directory(&context.temp_dir);

    let start_preprocessing = Instant::now();

    let progress_bar = PreprocessBar::new("Reprojecting to tiles".to_string());
    let tiles = reproject_to_tiles::<T>(src_dataset, context, Some(progress_bar.callback())).unwrap();
    progress_bar.finish();

    let progress_bar = PreprocessBar::new("Downsampling".to_string());
    let tiles = downsample_and_stitch::<T>(&tiles, context, Some(progress_bar.callback())).unwrap();
    progress_bar.finish();

    let progress_bar = PreprocessBar::new("Filling".to_string());
    create_mask_and_fill_no_data(&tiles, context, Some(progress_bar.callback())).unwrap();
    progress_bar.finish();

    delete_directory(&context.temp_dir);

    save_earth_config(tiles, context);

    println!("Preprocessing took: {:?}", start_preprocessing.elapsed());
}

pub fn preprocess(src_dataset: Dataset, context: &mut PreprocessContext) {
    macro_rules! preprocess_gen {
        ($data_type:ty) => {
            preprocess_gen::<$data_type>(src_dataset, context)
        };
    }

    match context.data_type {
        GdalDataType::Unknown => panic!("Unknown data type!"),
        GdalDataType::UInt8 => preprocess_gen!(u8),
        GdalDataType::UInt16 => preprocess_gen!(u16),
        GdalDataType::UInt32 => preprocess_gen!(u32),
        GdalDataType::UInt64 => preprocess_gen!(u64),
        GdalDataType::Int8 => preprocess_gen!(i8),
        GdalDataType::Int16 => preprocess_gen!(i16),
        GdalDataType::Int32 => preprocess_gen!(i32),
        GdalDataType::Int64 => preprocess_gen!(i64),
        GdalDataType::Float32 => preprocess_gen!(f32),
        GdalDataType::Float64 => preprocess_gen!(f64),
    };
}

/// Streaming version that processes tiles directly without creating full face images.
/// This uses significantly less memory and allows better parallelization.
pub fn preprocess_streaming(src_dataset: Dataset, context: &mut PreprocessContext) {
    macro_rules! preprocess_streaming_gen {
        ($data_type:ty) => {
            preprocess_streaming_gen::<$data_type>(src_dataset, context)
        };
    }

    match context.data_type {
        GdalDataType::Unknown => panic!("Unknown data type!"),
        GdalDataType::UInt8 => preprocess_streaming_gen!(u8),
        GdalDataType::UInt16 => preprocess_streaming_gen!(u16),
        GdalDataType::UInt32 => preprocess_streaming_gen!(u32),
        GdalDataType::UInt64 => preprocess_streaming_gen!(u64),
        GdalDataType::Int8 => preprocess_streaming_gen!(i8),
        GdalDataType::Int16 => preprocess_streaming_gen!(i16),
        GdalDataType::Int32 => preprocess_streaming_gen!(i32),
        GdalDataType::Int64 => preprocess_streaming_gen!(i64),
        GdalDataType::Float32 => preprocess_streaming_gen!(f32),
        GdalDataType::Float64 => preprocess_streaming_gen!(f64),
    };
}

fn save_earth_config(tiles: Vec<TileCoordinate>, context: &PreprocessContext) {
    let file_path = context.earth_path.join("config.tc.ron");

    let mut config = EarthConfig::load_file(&file_path).unwrap_or_default();

    config.shape = EarthShape::WGS84;
    config.path = context.earth_path.to_str().unwrap().to_string();
    config.add_attachment(context.attachment_label.clone(), context.attachment.clone());

    if context.attachment_label == AttachmentLabel::Topography {
        config.min_height = context.min_height;
        config.max_height = context.max_height;
        config.tiles = tiles;
        config.lod_count = context.lod_count.unwrap();
    }

    config.save_file(&file_path).unwrap();
}
