use image::{ImageBuffer, Luma};

// Scanline polygon fill algorithm
pub fn fill_polygon(
    img: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    vertices: &[(i32, i32)],
    color: Luma<u8>,
) {
    if vertices.len() < 3 {
        return;
    }

    let width = img.width() as i32;
    let height = img.height() as i32;

    // Find bounding box
    let min_y = vertices.iter().map(|(_, y)| *y).min().unwrap().max(0);
    let max_y = vertices
        .iter()
        .map(|(_, y)| *y)
        .max()
        .unwrap()
        .min(height - 1);

    // For each scanline
    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        // Find intersections with polygon edges
        for i in 0..vertices.len() {
            let j = (i + 1) % vertices.len();
            let (x1, y1) = vertices[i];
            let (x2, y2) = vertices[j];

            // Check if edge crosses scanline
            if (y1 <= y && y < y2) || (y2 <= y && y < y1) {
                if y1 != y2 {
                    // Calculate x coordinate of intersection
                    let x = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
                    intersections.push(x);
                }
            }
        }

        // Sort intersections
        intersections.sort_unstable();

        // Fill between pairs of intersections
        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                let x_start = chunk[0].max(0);
                let x_end = chunk[1].min(width - 1);

                for x in x_start..=x_end {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}
