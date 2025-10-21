use std::path::Path;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{args::{CliArgs, EarthCommand, ImageResolution}, processing::render_shapefile};

pub fn handle_distance_map(args: &CliArgs) {
    let EarthCommand::DistanceMap { input, output, resolutions } = &args.command else {
        unreachable!()
    };

    print!("Processing Distance Map...");

    resolutions.par_iter().for_each(|resolution| {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "distance_map.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match generate_distance_map(input, *resolution, &out_file_path) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling distance map to {}: {}", resolution, err),
        }
    });

    println!("");
}

fn generate_distance_map(
    input_file: &Path,
    resolution: ImageResolution,
    output_file: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let ocean_mask = render_shapefile(input_file, resolution, false)?;

    // Compute distance transform using Euclidean distance
    let distance_map = compute_distance_transform(&ocean_mask);

    // Save the output image
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    distance_map.save(output_file)?;

    Ok(())
}

/// Compute the Euclidean distance transform from an ocean mask.
/// Input: binary image where 0 = ocean, 255 = land
/// Output: grayscale image where pixel values represent distance to nearest ocean
fn compute_distance_transform(mask: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>) -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let width = mask.width();
    let height = mask.height();

    // Initialize distance map with max values
    let mut distances = vec![vec![f32::INFINITY; width as usize]; height as usize];

    // First pass: top-left to bottom-right
    for y in 0..height {
        for x in 0..width {
            let pixel = mask.get_pixel(x, y)[0];

            // If pixel is ocean (black), distance is 0
            if pixel == 0 {
                distances[y as usize][x as usize] = 0.0;
            } else {
                // Check neighbors
                let mut min_dist = f32::INFINITY;

                // Left neighbor
                if x > 0 {
                    min_dist = min_dist.min(distances[y as usize][x as usize - 1] + 1.0);
                }

                // Top neighbor
                if y > 0 {
                    min_dist = min_dist.min(distances[y as usize - 1][x as usize] + 1.0);
                }

                // Top-left diagonal
                if x > 0 && y > 0 {
                    min_dist = min_dist.min(distances[y as usize - 1][x as usize - 1] + std::f32::consts::SQRT_2);
                }

                // Top-right diagonal
                if x < width - 1 && y > 0 {
                    min_dist = min_dist.min(distances[y as usize - 1][x as usize + 1] + std::f32::consts::SQRT_2);
                }

                distances[y as usize][x as usize] = min_dist;
            }
        }
    }

    // Second pass: bottom-right to top-left
    for y in (0..height).rev() {
        for x in (0..width).rev() {
            let mut min_dist = distances[y as usize][x as usize];

            // Right neighbor
            if x < width - 1 {
                min_dist = min_dist.min(distances[y as usize][x as usize + 1] + 1.0);
            }

            // Bottom neighbor
            if y < height - 1 {
                min_dist = min_dist.min(distances[y as usize + 1][x as usize] + 1.0);
            }

            // Bottom-right diagonal
            if x < width - 1 && y < height - 1 {
                min_dist = min_dist.min(distances[y as usize + 1][x as usize + 1] + std::f32::consts::SQRT_2);
            }

            // Bottom-left diagonal
            if x > 0 && y < height - 1 {
                min_dist = min_dist.min(distances[y as usize + 1][x as usize - 1] + std::f32::consts::SQRT_2);
            }

            distances[y as usize][x as usize] = min_dist;
        }
    }

    // Find max distance for normalization
    let max_distance = distances.iter()
        .flat_map(|row| row.iter())
        .fold(0.0f32, |acc, &d| if d.is_finite() { acc.max(d) } else { acc });

    // Create output image, normalizing distances to 0-255 range
    let mut output = image::ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let dist = distances[y as usize][x as usize];
            let normalized = if dist.is_finite() && max_distance > 0.0 {
                ((dist / max_distance) * 255.0) as u8
            } else {
                0u8
            };
            output.put_pixel(x, y, image::Luma([normalized]));
        }
    }

    output
}