use bevy_color::prelude::*;

pub fn random_color() -> Color {
    bevy_color::Color::srgb(
        rand::random_range(0.2..0.8),
        rand::random_range(0.2..0.8),
        rand::random_range(0.2..0.8),
    )
}
