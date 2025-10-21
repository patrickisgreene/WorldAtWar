mod scale_image;
mod draw_line;
mod fill_polygon;
mod render_shapefile;

pub use draw_line::draw_line;
pub use scale_image::scale_image;
pub use fill_polygon::fill_polygon;
pub use render_shapefile::render_shapefile;

use std::path::Path;
use crate::args::ImageResolution;

pub fn generate_mask(
    input_file: &Path,
    resolution: ImageResolution,
    output_file: &Path,
    invert_color: bool
) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directories if they don't exist
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(render_shapefile(input_file, resolution, invert_color)?.save(output_file)?)
}

pub fn generate_scaled_image(
    input_file: &Path,
    resolution: ImageResolution,
    output_file: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    scale_image(input_file, resolution, output_file)
}