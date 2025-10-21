use bevy_ecs::prelude::*;
use bevy_math::UVec2;

use crate::geometry::CubeFace;

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Chunk {
    pub face: CubeFace,
    pub coord: UVec2,
    pub subdivision_level: u8,
}

impl Chunk {
    pub fn base(face: CubeFace) -> Chunk {
        Chunk {
            face,
            coord: Default::default(),
            subdivision_level: 0,
        }
    }

    pub fn child(chunk: &Chunk, x: u32, y: u32) -> Chunk {
        Chunk {
            face: chunk.face,
            coord: chunk.coord * 2 + UVec2::new(x, y),
            subdivision_level: chunk.subdivision_level + 1,
        }
    }

    pub fn get_children(&self) -> [Chunk; 4] {
        [
            Self::child(self, 0, 0),
            Self::child(self, 0, 1),
            Self::child(self, 1, 0),
            Self::child(self, 1, 1),
        ]
    }
}
