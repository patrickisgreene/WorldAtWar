use bevy_app::prelude::*;
use bevy_color::prelude::*;
use bevy_ecs::prelude::*;
use bevy_gizmos::prelude::*;
use bevy_math::prelude::*;
use bevy_transform::prelude::*;

use big_space::prelude::*;

#[derive(Component)]
pub struct RadarGizmoColor(pub Color);

/// Marker component to indicate that radar gizmos should be rendered for this entity
/// This is typically added/removed based on visibility or distance culling
#[derive(Component)]
pub struct RadarGizmoVisible;

#[derive(Component)]
pub struct Radar {
    pub strength: f32,
    pub rotation: Quat,
    pub translation: Vec3,
}

#[derive(Component)]
pub enum RadarShape {
    Cone { radius: f64, length: f64 },
    Sphere { radius: f64 },
}

pub struct RadarPlugin;

impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_radar_gizmos);
    }
}

fn draw_radar_gizmos(
    grids: Grids,
    mut gizmos: Gizmos,
    query: Query<
        (
            Entity,
            &Radar,
            &RadarShape,
            Option<&RadarGizmoColor>,
            &CellCoord,
            &Transform,
        ),
        With<RadarGizmoVisible>,
    >,
) {
    for (entity, radar, shape, color, cell, transform) in query {
        let color = color.map(|x| x.0).unwrap_or(Color::WHITE);
        let grid = grids.parent_grid(entity).unwrap();

        // Calculate the world position using grid coordinates
        let grid_position = grid.grid_position_double(cell, &transform);
        let position = grid_position + radar.translation.as_dvec3();

        match shape {
            RadarShape::Sphere { radius } => {
                // Draw a wireframe sphere
                gizmos.sphere(
                    position.as_vec3(),
                    //rotation,
                    *radius as f32,
                    color,
                );
            }
            RadarShape::Cone { radius, length } => {
                // Draw a cone gizmo with semi-transparent blue color
                // The cone points in the direction of the transform's forward vector
                let cone = Cone {
                    radius: *radius as f32,
                    height: *length as f32,
                };

                // Rotate 90 degrees on X axis so cone circle faces forward
                let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                let mut isometry = transform.to_isometry();
                isometry.rotation = isometry.rotation * rotation;

                // Set the world position (grid position + radar translation)
                isometry.translation = position.as_vec3().into();

                // Offset the cone so its point is at the entity position
                // The cone extends along its local +Y axis after rotation, so we offset by half its height
                let offset = isometry.rotation * bevy_math::Vec3::Y * (*length as f32 / 2.0);
                isometry.translation += bevy_math::Vec3A::from(-offset);

                gizmos.primitive_3d(&cone, isometry, Color::srgba(1.0, 0.0, 1.0, 1.0));
            }
        }
    }
}
