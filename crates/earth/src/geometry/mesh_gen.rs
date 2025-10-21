use super::{CHUNK_RESOLUTION, Chunk};
use bevy_asset::RenderAssetUsages;
use bevy_image::Image;
use bevy_math::{DVec3, prelude::*};
use bevy_mesh::{Indices, PrimitiveTopology, prelude::*};
use waw_utils::consts::EARTH_RADIUS;

/// Generate a mesh for a chunk of the cubesphere
/// Returns (mesh, chunk_center_position_global, rotation)
/// The mesh vertices are in local coordinates (centered at origin and rotated into local space)
/// The chunk_center is the global position where this chunk should be placed
/// The rotation aligns the local Y-axis with the chunk center direction
pub fn generate_chunk_mesh(chunk: &Chunk, heightmap: Option<&Image>) -> (Mesh, DVec3) {
    let resolution = CHUNK_RESOLUTION;
    let face = chunk.face;
    let coord = chunk.coord;
    let level = chunk.subdivision_level;

    // Calculate the size of this chunk on the cube face
    // Each subdivision level halves the size
    let chunks_per_edge = 1u32 << level; // 2^level
    let chunk_size = 2.0 / chunks_per_edge as f64; // Cube face spans -1 to 1

    // Calculate the offset of this chunk on the face (in normalized cube coordinates)
    let min_u = -1.0 + (coord.x as f64) * chunk_size;
    let min_v = -1.0 + (coord.y as f64) * chunk_size;

    let (tangent, bitangent) = face.local_axes();
    let normal = face.normal();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::with_capacity(((resolution - 1) * (resolution - 1) * 6) as usize);

    // Calculate chunk center first (center of face projected onto sphere)
    let center_u = tangent * (min_u + chunk_size / 2.0);
    let center_v = bitangent * (min_v + chunk_size / 2.0);
    let center_cube_pos = normal + center_u + center_v;
    let chunk_center = cube_to_sphere(center_cube_pos).normalize() * EARTH_RADIUS;

    // Generate vertices and store their data
    // Store intermediate data needed for proper T-junction fixing
    let mut vertex_grid = Vec::with_capacity((resolution * resolution) as usize);
    let mut cube_positions = Vec::with_capacity((resolution * resolution) as usize);

    for y in 0..resolution {
        for x in 0..resolution {
            // Calculate normalized position on the cube face
            let u = min_u + (x as f64 / (resolution - 1) as f64) * chunk_size;
            let v = min_v + (y as f64 / (resolution - 1) as f64) * chunk_size;

            // Project to cube face
            let cube_pos = normal + tangent * u + bitangent * v;
            cube_positions.push(cube_pos);

            // Apply cube-to-sphere mapping for even distribution
            let sphere_pos = cube_to_sphere(cube_pos).normalize() * EARTH_RADIUS;

            // Calculate normal (pointing outward from sphere center)
            let sphere_normal = cube_to_sphere(cube_pos).normalize().as_vec3();

            // Convert cube face position to equirectangular UV for NASA Blue Marble texture
            let texture_uv = cube_to_equirectangular_uv(cube_pos);

            let elevation = sample_texture(heightmap, texture_uv) as f64 * 500000.0;

            let final_pos = sphere_pos + sphere_normal.as_dvec3() * elevation;

            vertex_grid.push((
                (final_pos - chunk_center).as_vec3(),
                sphere_normal,
                texture_uv,
            ));
        }
    }

    // Generate indices and handle UV seam discontinuity
    let flip = face.flip_winding();
    for y in 0..(resolution - 1) {
        for x in 0..(resolution - 1) {
            let top_left_idx = (y * resolution + x) as usize;
            let top_right_idx = top_left_idx + 1;
            let bottom_left_idx = ((y + 1) * resolution + x) as usize;
            let bottom_right_idx = bottom_left_idx + 1;

            // Get UVs for this quad
            let quad_uvs = [
                vertex_grid[top_left_idx].2,
                vertex_grid[top_right_idx].2,
                vertex_grid[bottom_left_idx].2,
                vertex_grid[bottom_right_idx].2,
            ];

            // Check if this quad crosses the UV seam (longitude wraps from 1.0 to 0.0)
            // More robust check: if max and min U differ by more than 0.5, we've wrapped
            let min_u = quad_uvs
                .iter()
                .map(|uv| uv.x)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let max_u = quad_uvs
                .iter()
                .map(|uv| uv.x)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let crosses_seam = (max_u - min_u) > 0.5;

            // Helper to add vertex with seam handling
            let mut add_vertex = |vertex: (Vec3, Vec3, Vec2)| -> u32 {
                let (pos, normal, mut uv) = vertex;
                if crosses_seam && uv.x < 0.5 {
                    uv.x += 1.0;
                }
                positions.push(pos);
                normals.push(normal);
                uvs.push(uv);
                (positions.len() - 1) as u32
            };

            // Create vertex indices for this quad
            let tl = add_vertex(vertex_grid[top_left_idx]);
            let tr = add_vertex(vertex_grid[top_right_idx]);
            let bl = add_vertex(vertex_grid[bottom_left_idx]);
            let br = add_vertex(vertex_grid[bottom_right_idx]);

            // Normal quad - no T-junction
            if flip {
                indices.push(tl);
                indices.push(br);
                indices.push(bl);
                indices.push(tl);
                indices.push(tr);
                indices.push(br);
            } else {
                indices.push(tl);
                indices.push(bl);
                indices.push(br);
                indices.push(tl);
                indices.push(br);
                indices.push(tr);
            }
        }
    }

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices));

    (mesh, chunk_center)
}

fn sample_texture(image: Option<&Image>, uv: Vec2) -> f32 {
    if let Some(image) = image {
        let width = image.width() as f32;
        let height = image.height() as f32;

        // Wrap U coordinate for horizontal seam handling (repeat mode)
        let wrapped_u = uv.x.fract();

        // Convert UV coordinates to pixel coordinates
        let x = (wrapped_u * width).clamp(0.0, width - 1.0) as u32;
        let y = (uv.y * height).clamp(0.0, height - 1.0) as u32;

        if let Ok(color) = image.get_color_at(x, y) {
            return color.to_linear().red;
        }
    }
    0.0
}

pub fn cube_to_sphere(pos: DVec3) -> DVec3 {
    let x2: f64 = pos.x * pos.x;
    let y2: f64 = pos.y * pos.y;
    let z2: f64 = pos.z * pos.z;
    DVec3::new(
        pos.x * (1.0f64 - y2 / 2.0f64 - z2 / 2.0f64 + y2 * z2 / 3.0f64).sqrt(),
        pos.y * (1.0f64 - x2 / 2.0f64 - z2 / 2.0f64 + x2 * z2 / 3.0f64).sqrt(),
        pos.z * (1.0f64 - x2 / 2.0f64 - y2 / 2.0f64 + x2 * y2 / 3.0f64).sqrt(),
    )
}

/// Maps cube position to equirectangular UV for NASA Blue Marble texture
/// Uses the cube-to-sphere mapped position to calculate lat/lon
pub fn cube_to_equirectangular_uv(cube_pos: DVec3) -> Vec2 {
    // Apply cube-to-sphere mapping to get uniform distribution
    let sphere_pos = cube_to_sphere(cube_pos).normalize();

    // Convert to spherical coordinates (lat/lon)
    // atan2(z, x) gives longitude, asin(y) gives latitude
    let lon = sphere_pos.z.atan2(sphere_pos.x);
    let lat = sphere_pos.y.asin();

    // Map to UV coordinates [0, 1]
    // Longitude: -π to π → 0 to 1
    // Latitude: -π/2 to π/2 → 0 to 1
    let u = 1.0 - ((lon / std::f64::consts::TAU) + 0.5) as f32;
    let v = 1.0 - ((lat / std::f64::consts::PI) + 0.5) as f32;

    Vec2::new(u, v)
}
