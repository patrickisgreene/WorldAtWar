mod chunk;
mod face;
mod heightmap;
mod mesh_gen;

pub use chunk::Chunk;
pub use face::CubeFace;
pub use heightmap::{EarthHeightMap, EarthUrbanAreas};
pub use mesh_gen::{cube_to_equirectangular_uv, cube_to_sphere, generate_chunk_mesh};

const CHUNK_RESOLUTION: u32 = 16;
