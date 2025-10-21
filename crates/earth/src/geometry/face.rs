use bevy_ecs::prelude::*;
use bevy_math::DVec3;

#[derive(Component, Default, Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum CubeFace {
    #[default]
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl CubeFace {
    pub const ALL: [Self; 6] = [
        CubeFace::Front,
        CubeFace::Back,
        CubeFace::Left,
        CubeFace::Right,
        CubeFace::Top,
        CubeFace::Bottom,
    ];

    pub fn normal(&self) -> DVec3 {
        match self {
            CubeFace::Back => DVec3::Z,
            CubeFace::Front => DVec3::NEG_Z,
            CubeFace::Top => DVec3::Y,
            CubeFace::Bottom => DVec3::NEG_Y,
            CubeFace::Right => DVec3::X,
            CubeFace::Left => DVec3::NEG_X,
        }
    }

    /// Get the local coordinate axes for this face
    /// Returns (tangent, bitangent) vectors
    pub fn local_axes(&self) -> (DVec3, DVec3) {
        match self {
            CubeFace::Front => (DVec3::X, DVec3::Y),
            CubeFace::Back => (DVec3::NEG_X, DVec3::Y),
            CubeFace::Top => (DVec3::X, DVec3::NEG_Z),
            CubeFace::Bottom => (DVec3::X, DVec3::Z),
            CubeFace::Right => (DVec3::NEG_Z, DVec3::Y),
            CubeFace::Left => (DVec3::Z, DVec3::Y),
        }
    }

    /// Returns true if triangle winding order should be flipped for this face
    /// to ensure normals point outward from the sphere
    pub fn flip_winding(&self) -> bool {
        match self {
            CubeFace::Front | CubeFace::Back => false,
            CubeFace::Left | CubeFace::Right | CubeFace::Top | CubeFace::Bottom => true,
        }
    }
}
