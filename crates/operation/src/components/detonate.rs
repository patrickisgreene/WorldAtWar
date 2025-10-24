use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_math::prelude::*;
use bevy_time::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct Detonation {
    pub effect: Handle<EffectAsset>,
    pub timer: Timer,
}

impl Detonation {
    pub fn new(asset_server: &AssetServer) -> Detonation {
        // Timer duration should match the particle lifetime (2 seconds)
        // Add a small buffer to ensure all particles have fully expired
        let timer = Timer::new(Duration::from_secs_f32(2.1), TimerMode::Once);

        Detonation {
            effect: asset_server.add(create_explosion_effect()),
            timer,
        }
    }
}

fn create_explosion_effect() -> EffectAsset {
    // Define a color gradient: bright yellow/orange -> red -> dark smoke
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(2.0, 1.5, 0.5, 1.0)); // Very bright yellow-orange flash
    color_gradient.add_key(0.3, Vec4::new(1.5, 0.5, 0.1, 1.0)); // Orange-red
    color_gradient.add_key(0.7, Vec4::new(0.8, 0.2, 0.05, 0.6)); // Dark red
    color_gradient.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.0)); // Smoke fading out

    // Size gradient: Much larger particles for visibility from space
    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(50000.0)); // Start at 50km
    size_gradient.add_key(0.3, Vec3::splat(100000.0)); // Expand rapidly to 100km
    size_gradient.add_key(1.0, Vec3::splat(150000.0)); // End at 150km

    // Create a new expression module
    let mut module = Module::default();

    // Smaller initial spawn radius - particles will expand via velocity and size
    let explosion_radius = 5000.0; // Start at 10km radius

    // Spawn particles on the surface of a sphere
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(explosion_radius),
        dimension: ShapeDimension::Surface,
    };

    // Fast radial velocity to make the explosion expand rapidly
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(100000.0), // Fixed high speed for dramatic expansion
    };

    // Longer lifetime to keep explosion visible
    let lifetime = module.lit(2.0); // 5 seconds
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // No acceleration - just radial expansion
    let accel = module.lit(Vec3::ZERO);
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset with burst spawner - MUCH FEWER PARTICLES for performance
    EffectAsset::new(1000, SpawnerSettings::once(1000.0.into()), module)
        .with_name("Detenation")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .update(update_accel)
        // Render the particles with a color gradient over their lifetime
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            ..Default::default()
        })
        // Render with size over lifetime - THIS IS CRITICAL for visibility!
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
}
