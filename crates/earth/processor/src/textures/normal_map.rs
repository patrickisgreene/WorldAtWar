use std::path::Path;

use crate::args::{CliArgs, EarthCommand, ImageResolution};

pub fn handle_normal_map(args: &CliArgs) {
    let EarthCommand::NormalMap {
        input,
        output,
        resolutions,
    } = &args.command
    else {
        unreachable!()
    };

    print!("Processing normal map...");

    for resolution in resolutions {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "normal_map.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        if let Err(err) = generate_normal_map(input, *resolution, &out_file_path) {
            eprintln!("Error generating normal map at {}: {}", resolution, err);
        }
        print!("✔");
    }
    println!("");
}

fn generate_normal_map(
    input_file: &Path,
    resolution: ImageResolution,
    output_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{GenericImageView, ImageBuffer, Pixel, Rgb};

    // Open the input heightmap
    let heightmap = image::open(input_file)?;
    let (src_width, src_height) = heightmap.dimensions();

    // Define tile size - process the image in chunks to avoid memory issues
    const TILE_SIZE: u32 = 512;
    let tile_cols = (resolution.width + TILE_SIZE - 1) / TILE_SIZE;
    let tile_rows = (resolution.height + TILE_SIZE - 1) / TILE_SIZE;

    // Create output image buffer
    let mut output = ImageBuffer::new(resolution.width, resolution.height);

    // Process each tile
    let tiles: Vec<(u32, u32)> = (0..tile_rows)
        .flat_map(|row| (0..tile_cols).map(move |col| (col, row)))
        .collect();

    for (tile_col, tile_row) in tiles {
        let tile_x = tile_col * TILE_SIZE;
        let tile_y = tile_row * TILE_SIZE;
        let tile_width = TILE_SIZE.min(resolution.width - tile_x);
        let tile_height = TILE_SIZE.min(resolution.height - tile_y);

        // Process this tile with overlap for edge calculations
        let overlap = 1;
        let extended_x = tile_x.saturating_sub(overlap);
        let extended_y = tile_y.saturating_sub(overlap);
        let extended_width = (tile_width + 2 * overlap).min(resolution.width - extended_x);
        let extended_height = (tile_height + 2 * overlap).min(resolution.height - extended_y);

        // Sample the heightmap for this extended tile
        let mut tile_heights =
            vec![vec![0.0f32; extended_width as usize]; extended_height as usize];
        for ty in 0..extended_height {
            for tx in 0..extended_width {
                let out_x = extended_x + tx;
                let out_y = extended_y + ty;

                // Map output coordinates back to source image
                let src_x = ((out_x as f64 / resolution.width as f64) * src_width as f64) as u32;
                let src_y = ((out_y as f64 / resolution.height as f64) * src_height as f64) as u32;
                let src_x = src_x.min(src_width - 1);
                let src_y = src_y.min(src_height - 1);

                // Get height value (grayscale from heightmap)
                let pixel = heightmap.get_pixel(src_x, src_y);
                let height = pixel.to_luma()[0] as f32 / 255.0;
                tile_heights[ty as usize][tx as usize] = height;
            }
        }

        // Calculate normals for the actual tile area (not the overlap)
        let x_offset = (tile_x - extended_x) as usize;
        let y_offset = (tile_y - extended_y) as usize;

        for ty in 0..tile_height {
            for tx in 0..tile_width {
                let local_x = x_offset + tx as usize;
                let local_y = y_offset + ty as usize;

                // Calculate normal using Sobel operator
                let strength = 8.0; // Adjust this to control normal map intensity

                let tl = tile_heights[local_y.saturating_sub(1)][local_x.saturating_sub(1)];
                let t = tile_heights[local_y.saturating_sub(1)][local_x];
                let tr = tile_heights[local_y.saturating_sub(1)]
                    [(local_x + 1).min(extended_width as usize - 1)];
                let l = tile_heights[local_y][local_x.saturating_sub(1)];
                let r = tile_heights[local_y][(local_x + 1).min(extended_width as usize - 1)];
                let bl = tile_heights[(local_y + 1).min(extended_height as usize - 1)]
                    [local_x.saturating_sub(1)];
                let b = tile_heights[(local_y + 1).min(extended_height as usize - 1)][local_x];
                let br = tile_heights[(local_y + 1).min(extended_height as usize - 1)]
                    [(local_x + 1).min(extended_width as usize - 1)];

                // Sobel filter
                let dx = (tr + 2.0 * r + br) - (tl + 2.0 * l + bl);
                let dy = (bl + 2.0 * b + br) - (tl + 2.0 * t + tr);

                // Create normal vector
                let nx = -dx * strength;
                let ny = -dy * strength;
                let nz = 1.0;

                // Normalize
                let len = (nx * nx + ny * ny + nz * nz).sqrt();
                let nx = nx / len;
                let ny = ny / len;
                let nz = nz / len;

                // Convert to RGB (0-255 range)
                let r = ((nx * 0.5 + 0.5) * 255.0) as u8;
                let g = ((ny * 0.5 + 0.5) * 255.0) as u8;
                let b = ((nz * 0.5 + 0.5) * 255.0) as u8;

                output.put_pixel(tile_x + tx, tile_y + ty, Rgb([r, g, b]));
            }
        }
    }

    // Create parent directories if they don't exist
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    output.save(output_file)?;

    Ok(())
}
