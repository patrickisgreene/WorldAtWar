mod chunk;
mod face;
mod heightmap;
mod mesh_gen;

pub use chunk::Chunk;
pub use face::CubeFace;
pub use heightmap::{EarthHeightMap, EarthUrbanAreas};
pub use mesh_gen::{generate_chunk_mesh, cube_to_sphere, cube_to_equirectangular_uv};

const CHUNK_RESOLUTION: u32 = 16;
