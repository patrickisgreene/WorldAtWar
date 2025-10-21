use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_math::{DVec3, prelude::*};
use bevy_transform::prelude::*;
use big_space::prelude::*;

use crate::EarthLevelOfDetailFocus;

#[derive(SystemParam)]
pub struct CameraQuery<'w, 's> {
    grid: Query<'w, 's, (Entity, &'static Grid)>,
    camera: Query<'w, 's, (&'static CellCoord, &'static Transform), With<EarthLevelOfDetailFocus>>,
}

impl<'w, 's> CameraQuery<'w, 's> {
    pub fn grid(&self) -> &Grid {
        self.grid.single().unwrap().1
    }

    pub fn full_grid(&self) -> (Entity, &Grid) {
        self.grid.single().unwrap()
    }

    pub fn camera_position(&self) -> bevy_math::DVec3 {
        let Ok((_, grid)) = self.grid.single() else {
            panic!("Unable to run subdivision system: No Grid");
        };
        let Ok((cam_coord, cam_trans)) = self.camera.single() else {
            panic!("Unable to run subdivision system: No Camera");
        };
        grid.grid_position_double(cam_coord, cam_trans)
    }

    pub fn get_offset(&self, center: DVec3) -> (CellCoord, Vec3) {
        let (_, grid) = self.full_grid();
        grid.translation_to_grid(center)
    }

    pub fn world_position(&self, coord: &CellCoord, trans: &Transform) -> DVec3 {
        let (_, grid) = self.full_grid();
        grid.grid_position_double(coord, trans)
    }
}
