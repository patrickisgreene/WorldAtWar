use bevy::{prelude::*, render::{render_resource::AsBindGroup}, shader::ShaderRef};

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct EarthMaterial {
    #[uniform(0)]
    pub shallow_water_color: LinearRgba,
    #[uniform(0)]
    pub medium_water_color: LinearRgba,
    #[uniform(0)]
    pub deep_water_color: LinearRgba,
    #[uniform(0)]
    pub ripple_speed: f32,
    #[uniform(0)]
    pub ripple_frequency: f32,
    #[uniform(0)]
    pub ripple_distance: f32,

    #[texture(1)]
    #[sampler(2)]
    pub water_normal: Handle<Image>
}

impl Material for EarthMaterial {
    fn fragment_shader() -> ShaderRef {
        "earth/shader.wgsl".into()
    }
}