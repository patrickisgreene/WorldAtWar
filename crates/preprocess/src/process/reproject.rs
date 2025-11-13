use crate::core::CustomTransformer;
use crate::core::{
    CountingProgressCallback, FaceInfo, GDALCustomTransformer, PreprocessContext, PreprocessError,
    PreprocessResult, ProgressCallback, SuggestedWarpOutput, create_empty_dataset,
    create_tile_dataset, warp,
};
use gdal::{Dataset, GeoTransform, GeoTransformEx, Metadata, raster::{Buffer, GdalType}};
use glam::{DVec2, IVec2, U64Vec2};
use itertools::{Itertools, iproduct};
use num::NumCast;
use rayon::prelude::*;
use std::collections::HashMap;
use waw_earth::{math::TileCoordinate, prelude::AttachmentLabel};

pub struct Transform<'a> {
    pub transformer: GDALCustomTransformer,
    pub face: u32,
    pub lod: u32,
    pub size: U64Vec2,
    pub geo_transform: GeoTransform,
    pub uv_start: DVec2,
    pub uv_end: DVec2,
    pub pixel_start: IVec2,
    pub pixel_end: IVec2,
    pub progress_callback: Option<Box<ProgressCallback<'a>>>,
}

pub fn reproject<T: Copy + GdalType>(
    src_dataset: Dataset,
    context: &mut PreprocessContext,
    progress_callback: Option<&ProgressCallback>,
) -> PreprocessResult<HashMap<u32, FaceInfo>> {
    if let Some(progress_callback) = progress_callback {
        progress_callback(0.0);
    }

    let mut transforms = compute_transforms(&src_dataset, context, progress_callback)?;

    let faces = transforms
        .iter_mut()
        .map(|transform| {
            let dst_path = context.temp_dir.join(format!("face{}.tif", transform.face));
            let dst_dataset = create_empty_dataset::<T>(
                &dst_path,
                transform.size,
                Some(transform.geo_transform),
                context,
            )?;

            warp(
                &src_dataset,
                &dst_dataset,
                context,
                &mut transform.transformer,
                transform.progress_callback.as_deref(),
            )?;

            if matches!(context.attachment_label, AttachmentLabel::Topography) {
                let min_max = dst_dataset
                    .rasterband(1)
                    .unwrap()
                    .compute_raster_min_max(true)
                    .unwrap();

                context.min_height = context.min_height.min(min_max.min as f32);
                context.max_height = context.max_height.max(min_max.max as f32);
            }

            Ok((
                transform.face,
                FaceInfo {
                    lod: transform.lod,
                    pixel_start: transform.pixel_start,
                    pixel_end: transform.pixel_end,
                    path: dst_path,
                },
            ))
        })
        .collect::<PreprocessResult<HashMap<_, _>>>()?;

    Ok(faces)
}

/// Reproject directly to tiles without creating intermediate face images
pub fn reproject_to_tiles<T: Copy + GdalType + PartialEq + NumCast + Send + Sync>(
    src_dataset: Dataset,
    context: &mut PreprocessContext,
    progress_callback: Option<&ProgressCallback>,
) -> PreprocessResult<Vec<TileCoordinate>> {
    use crate::core::SharedReadOnlyDataset;
    use std::sync::Arc;
    use std::path::PathBuf;

    if let Some(progress_callback) = progress_callback {
        progress_callback(0.0);
    }

    let transforms = compute_transforms(&src_dataset, context, None)?;

    // Get the source path from the dataset description (file path)
    let src_path_str = src_dataset.description()?;
    let src_path = PathBuf::from(src_path_str);

    drop(src_dataset);

    let shared_src = Arc::new(SharedReadOnlyDataset::new(&src_path));

    // Compute all tiles that need to be generated
    let mut input_tiles = Vec::new();
    let mut face_infos = HashMap::new();

    for transform in &transforms {
        let xy_start = transform.pixel_start / context.attachment.center_size() as i32;
        let xy_end = (transform.pixel_end - 1) / context.attachment.center_size() as i32 + 1;

        face_infos.insert(
            transform.face,
            FaceInfo {
                lod: transform.lod,
                pixel_start: transform.pixel_start,
                pixel_end: transform.pixel_end,
                path: Default::default(), // Not needed for tile-based approach
            },
        );

        input_tiles.extend(
            iproduct!(xy_start.x..xy_end.x, xy_start.y..xy_end.y)
                .map(|(x, y)| TileCoordinate::new(transform.face, transform.lod, IVec2::new(x, y))),
        );
    }

    let count = input_tiles.len() as u64;
    let progress = CountingProgressCallback::new(count, progress_callback);

    // Process tiles in parallel
    let output_tiles = input_tiles
        .par_iter()
        .map(|&tile_coordinate| {
            let src_dataset = shared_src.get();
            let face_info = face_infos.get(&tile_coordinate.face).unwrap();

            // Create a new transformer for this tile
            let tile_pixel_start = tile_coordinate.xy * context.attachment.center_size() as i32;
            let tile_pixel_end = (tile_coordinate.xy + 1) * context.attachment.center_size() as i32;

            let copy_size =
                tile_pixel_end.min(face_info.pixel_end) - tile_pixel_start.max(face_info.pixel_start);
            let tile_offset = (face_info.pixel_start - tile_pixel_start).max(IVec2::ZERO)
                + context.attachment.border_size as i32;

            // Only process if there's data in this tile region
            if copy_size.x <= 0 || copy_size.y <= 0 {
                progress.increment();
                return Ok::<Option<TileCoordinate>, PreprocessError>(None);
            }

            // Compute geo_transform for this specific tile region in global S2 space
            let max_lod = context.lod_count.unwrap() - 1;
            let pixel_size = 1.0 / ((1 << max_lod) * context.attachment.center_size()) as f64;

            // The warp region in global pixel coordinates
            let warp_pixel_start = tile_pixel_start.max(face_info.pixel_start);
            let warp_pixel_end = tile_pixel_end.min(face_info.pixel_end);
            let warp_size = warp_pixel_end - warp_pixel_start;

            // Create a temporary dataset for just this warp region
            // The geo_transform maps the temp dataset's pixels to S2 UV space
            let warp_geo_transform = GeoTransform::from([
                warp_pixel_start.x as f64 * pixel_size,  // UV x origin
                pixel_size,                               // UV x resolution
                0.0,
                warp_pixel_start.y as f64 * pixel_size,  // UV y origin
                0.0,
                pixel_size,                               // UV y resolution
            ]);

            let temp_path = std::env::temp_dir().join(format!("temp_tile_{}_{}_{}_{}_{}.tif",
                std::process::id(), tile_coordinate.face, tile_coordinate.lod,
                tile_coordinate.xy.x, tile_coordinate.xy.y));
            let temp_dataset = create_empty_dataset::<T>(
                &temp_path,
                U64Vec2::new(warp_size.x as u64, warp_size.y as u64),
                Some(warp_geo_transform),
                context,
            )?;

            // Create transformer for this specific region
            let mut transformer = CustomTransformer::from_dataset(
                src_dataset,
                tile_coordinate.face,
                Some(warp_geo_transform),
            )?;

            // Warp the source data into the temporary dataset
            warp(src_dataset, &temp_dataset, context, &mut transformer, None)?;

            // Check if tile has any actual data (not just no-data values)
            let mut has_data = false;
            let copy_buffers: Vec<Buffer<T>> = temp_dataset
                .rasterbands()
                .map(|raster| {
                    let raster = raster?;
                    let buffer = raster.read_as::<T>(
                        (0, 0),
                        (warp_size.x as usize, warp_size.y as usize),
                        (warp_size.x as usize, warp_size.y as usize),
                        None,
                    )?;

                    let no_data_value = raster
                        .no_data_value()
                        .map(|v| T::from(v).ok_or(PreprocessError::NoDataOutOfRange))
                        .transpose()?;

                    has_data |= no_data_value.is_none()
                        || buffer
                            .data()
                            .iter()
                            .any(|&value| value != no_data_value.unwrap());

                    Ok::<Buffer<T>, PreprocessError>(buffer)
                })
                .try_collect()?;

            // Only create the tile if it has data
            if has_data {
                let tile_dataset = create_tile_dataset::<T>(tile_coordinate, context)?;

                for (band_index, mut copy_buffer) in copy_buffers.into_iter().enumerate() {
                    let mut tile_raster = tile_dataset.rasterband(band_index + 1)?;

                    tile_raster.write::<T>(
                        (tile_offset.x as isize, tile_offset.y as isize),
                        (warp_size.x as usize, warp_size.y as usize),
                        &mut copy_buffer,
                    )?;
                }
            }

            // Clean up temp file
            drop(temp_dataset);
            let _ = std::fs::remove_file(&temp_path);

            progress.increment();

            Ok::<Option<TileCoordinate>, PreprocessError>(has_data.then_some(tile_coordinate))
        })
        .filter_map(Result::transpose)
        .collect::<PreprocessResult<Vec<TileCoordinate>>>()?;

    // Update min/max height if needed
    if matches!(context.attachment_label, AttachmentLabel::Topography) {
        for tile_coord in &output_tiles {
            if let Ok(Some(dataset)) = crate::core::load_tile_dataset_if_exists(*tile_coord, context) {
                if let Ok(min_max) = dataset.rasterband(1).unwrap().compute_raster_min_max(true) {
                    context.min_height = context.min_height.min(min_max.min as f32);
                    context.max_height = context.max_height.max(min_max.max as f32);
                }
            }
        }
    }

    Ok(output_tiles)
}

pub fn compute_transforms<'a>(
    src_dataset: &Dataset,
    context: &mut PreprocessContext,
    progress_callback: Option<&'a ProgressCallback>,
) -> PreprocessResult<Vec<Transform<'a>>> {
    let mut transforms = Vec::with_capacity(6);

    let mut total_area = 0.0;

    for face in 0..6 {
        let mut transformer = CustomTransformer::from_dataset(src_dataset, face, None)?;

        let Some(SuggestedWarpOutput {
            size,
            mut geo_transform,
        }) = SuggestedWarpOutput::compute(src_dataset, &mut transformer)?
        else {
            continue;
        };

        // flip y axis
        geo_transform[3] += geo_transform[5] * size.y as f64;
        geo_transform[5] = -geo_transform[5];

        let uv_start = DVec2::from(geo_transform.apply(0.0, 0.0)).max(DVec2::ZERO);
        let uv_end = DVec2::from(geo_transform.apply(size.x as f64, size.y as f64)).min(DVec2::ONE);

        total_area += (uv_end - uv_start).element_product();

        transforms.push(Transform {
            face,
            size,
            uv_start,
            uv_end,
            lod: 0,
            pixel_start: IVec2::ZERO,
            pixel_end: IVec2::ZERO,
            transformer,
            geo_transform,
            progress_callback: None,
        });
    }

    let max_lod = if let Some(lod_count) = context.lod_count {
        lod_count - 1
    } else {
        let mut max_lod = 0;

        for transform in &mut transforms {
            // GDAL uses a heuristic to compute the output dimensions in pixels by setting
            // the same number of pixels on the diagonal on both the input and output
            // projections. Since we have up to six different output images, this
            // heuristic must be modified a bit. Since the S2 projection with a
            // quadratic mapping is quite area-uniform, we divide the total GDAL based
            // output image into the six output images by their area proportion of the total
            // output.

            let uv_size = transform.uv_end - transform.uv_start;

            let correction = uv_size.element_product().sqrt() / total_area.sqrt();
            let size = (transform.size.as_dvec2() * correction).round();

            max_lod = max_lod.max(
                (size / context.attachment.center_size() as f64 / uv_size)
                    .max_element()
                    .log2()
                    .ceil() as u32,
            );
        }

        context.lod_count = Some(max_lod + 1);

        max_lod
    };

    let pixel_size = 1.0 / ((1 << max_lod) * context.attachment.center_size()) as f64;

    for transform in &mut transforms {
        let pixel_start = (transform.uv_start / pixel_size).floor();
        let pixel_end = (transform.uv_end / pixel_size).ceil();

        transform.lod = max_lod;
        transform.size = (pixel_end - pixel_start).as_u64vec2();
        transform.geo_transform = GeoTransform::from([
            pixel_start.x * pixel_size,
            pixel_size,
            0.0,
            pixel_start.y * pixel_size,
            0.0,
            pixel_size,
        ]);
        transform.pixel_start = pixel_start.as_ivec2();
        transform.pixel_end = pixel_end.as_ivec2();
        transform.transformer =
            CustomTransformer::from_dataset(src_dataset, transform.face, Some(transform.geo_transform))?;
    }

    let work_portions = transforms
        .iter()
        .map(|transform| transform.size.element_product())
        .collect_vec();
    let total_work = work_portions.iter().sum::<u64>() as f64;
    let callback_intervals = work_portions
        .iter()
        .scan(0, |work_done, &work_portion| {
            *work_done += work_portion;
            Some((
                (*work_done - work_portion) as f64 / total_work,
                work_portion as f64 / total_work,
            ))
        })
        .collect_vec();

    for (transform, (offset, scale)) in transforms.iter_mut().zip(callback_intervals) {
        transform.progress_callback = progress_callback.map(|progress_callback| {
            Box::new(move |completion: f64| {
                progress_callback(completion.clamp(0.0, 1.0).mul_add(scale, offset))
            }) as Box<ProgressCallback>
        })
    }

    Ok(transforms)
}
