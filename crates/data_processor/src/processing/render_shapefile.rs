use std::path::Path;

use image::Luma;
use std::error::Error;
use shapefile::Reader;
use image::ImageBuffer;
use crate::args::ImageResolution;
use super::{draw_line, fill_polygon};

pub fn render_shapefile(input_file: &Path, resolution: ImageResolution, invert_color: bool) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, Box<dyn Error>> {
    let bg_color = Luma([if invert_color { 0u8 } else { 255u8 }]);
    // Create a white image (background)
    let mut img = ImageBuffer::from_pixel(resolution.width, resolution.height, bg_color);

    // Read the shapefile
    let mut reader = Reader::from_path(input_file)?;

    // Iterate through all shapes
    for shape_record in reader.iter_shapes_and_records() {
        let (shape, _record) = shape_record?;

        let color = Luma([if invert_color { 255u8 } else { 0u8 }]);

        // Convert shape coordinates to pixel coordinates
        // Assuming WGS84 coordinate system: longitude [-180, 180], latitude [-90, 90]
        match shape {
            shapefile::Shape::Polygon(polygon) => {
                for ring in polygon.rings() {
                    // Convert points to pixel coordinates
                    let pixels: Vec<(i32, i32)> = ring.points()
                        .iter()
                        .map(|point| {
                            let x = ((point.x + 180.0) / 360.0 * resolution.width as f64) as i32;
                            let y = ((90.0 - point.y) / 180.0 * resolution.height as f64) as i32;
                            (x, y)
                        })
                        .collect();

                    // Fill the polygon using scanline algorithm
                    fill_polygon(&mut img, &pixels, color);
                }
            },
            shapefile::Shape::Polyline(polyline) => {
                for part in polyline.parts() {
                    let points: Vec<_> = part.into_iter().collect();
                    for window in points.windows(2) {
                        let p1 = &window[0];
                        let p2 = &window[1];

                        let x1 = ((p1.x + 180.0) / 360.0 * resolution.width as f64) as i32;
                        let y1 = ((90.0 - p1.y) / 180.0 * resolution.height as f64) as i32;
                        let x2 = ((p2.x + 180.0) / 360.0 * resolution.width as f64) as i32;
                        let y2 = ((90.0 - p2.y) / 180.0 * resolution.height as f64) as i32;

                        // Draw line between consecutive points
                        draw_line(&mut img, x1, y1, x2, y2, color);
                    }
                }
            },
            shapefile::Shape::Point(point) => {
                let x = ((point.x + 180.0) / 360.0 * resolution.width as f64) as i32;
                let y = ((90.0 - point.y) / 180.0 * resolution.height as f64) as i32;

                if x >= 0 && x < resolution.width as i32 && y >= 0 && y < resolution.height as i32 {
                    img.put_pixel(x as u32, y as u32, color);
                }
            },
            shapefile::Shape::Multipoint(multipoint) => {
                for point in multipoint.points() {
                    let x = ((point.x + 180.0) / 360.0 * resolution.width as f64) as i32;
                    let y = ((90.0 - point.y) / 180.0 * resolution.height as f64) as i32;

                    if x >= 0 && x < resolution.width as i32 && y >= 0 && y < resolution.height as i32 {
                        img.put_pixel(x as u32, y as u32, color);
                    }
                }
            },
            _ => {} // Handle other shape types if needed
        }
    }

    Ok(img)
}