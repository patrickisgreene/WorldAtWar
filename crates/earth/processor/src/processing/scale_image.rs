use std::path::Path;

use image::imageops::FilterType;
use image::{ImageReader, ImageDecoder};

use crate::args::ImageResolution;

pub fn scale_image(
    input_file: &Path,
    resolution: ImageResolution,
    output_file: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    // Scope the image loading and processing to ensure memory is freed
    {
        // Load the input image with disabled memory limits
        let reader = ImageReader::open(input_file)?;
        let mut decoder = reader.into_decoder()?;

        // Disable memory limits to allow processing very large images
        decoder.set_limits(image::Limits::no_limits())?;

        let img = image::DynamicImage::from_decoder(decoder)?;

        // Use Triangle filter for better performance (3-5x faster than Lanczos3)
        // Triangle (bilinear) provides good quality/speed balance
        // For maximum quality use Lanczos3, for maximum speed use Nearest
        let resized = img.resize_exact(
            resolution.width,
            resolution.height,
            FilterType::Triangle
        );

        // Create parent directories if they don't exist
        if let Some(parent) = output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save the resized image as PNG
        resized.save(output_file)?;

        // img and resized are dropped here when scope ends
    }

    Ok(())
}