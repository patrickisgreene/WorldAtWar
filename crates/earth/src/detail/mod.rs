mod camera;
mod graph;

pub use camera::CameraQuery;
pub use graph::*;
use waw_utils::consts::GEOSYNCHRONUS_ORBIT;

pub const CHILDREN_PER_CHUNK: usize = 4;

pub fn distance_to_lod_level(distance: f64) -> u8 {
    const MAX_SUBDIVISIONS: u8 = 13;

    // Discrete distance thresholds that decrease exponentially
    // Each level requires roughly half the distance of the previous
    let level = if distance >= GEOSYNCHRONUS_ORBIT {
        0 // Beyond GEO: lowest detail
    } else if distance >= GEOSYNCHRONUS_ORBIT * 0.25 {
        1 // High orbit (0.5-1.0 GEO)
    } else if distance >= GEOSYNCHRONUS_ORBIT * 0.15 {
        2 // Medium-high orbit (0.1-0.5 GEO)
    } else if distance >= GEOSYNCHRONUS_ORBIT * 0.1 {
        3 // Low orbit (0.01-0.1 GEO, ~4,200-42,000 km)
    } else if distance >= GEOSYNCHRONUS_ORBIT * 0.05 {
        4 // Very low orbit (420-4,200 km)
    } else if distance >= GEOSYNCHRONUS_ORBIT * 0.025 {
        5 // Near surface (42-420 km)
    } else {
        6 // Surface/atmosphere (<42 km)
    };

    level.min(MAX_SUBDIVISIONS)
}
