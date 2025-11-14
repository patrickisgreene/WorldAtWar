#import waw_earth::types::{EarthMaterial, AtlasTile}
#import waw_earth::bindings::{
    bathyometry_attachment,
    earth, earth_view, nighttime_attachment,
    attachments, daytime_attachment, ocean_attachment
}
#import bevy_pbr::mesh_view_bindings::{globals, view}
#import waw_earth::attachments::{
    compute_sample_uv, sample_height,
    sample_height_mask,
    sample_surface_gradient, relief_shading
}
#import waw_earth::fragment::{
    FragmentInput, FragmentOutput, FragmentInfo,
    fragment_info, fragment_debug
}
#import bevy_pbr::pbr_functions::{calculate_view, apply_pbr_lighting}
#import bevy_pbr::pbr_types::{PbrInput, pbr_input_new}
#import waw_earth::functions::{apply_height, lookup_tile, sample_attachment}

fn grayscale(color: vec3<f32>) -> f32 {
    return (0.299 * color.x) + (0.587 * color.y) + (0.114 * color.z);
}

@group(3) @binding(0)
var<uniform> material: EarthMaterial;
@group(3) @binding(1) var water_normal_texture: texture_2d<f32>;
@group(3) @binding(2) var water_normal_sampler: sampler;

fn terrain(tile: AtlasTile, info: FragmentInfo) -> vec3<f32> {
    let world_position = vec4<f32>(apply_height(info.world_coordinate, info.height), 1.0);
        let surface_gradient = sample_surface_gradient(tile, info.tangent_space);

    var pbr_input: PbrInput                 = pbr_input_new();
    pbr_input.material.base_color           = vec4<f32>(1.0);
    pbr_input.material.perceptual_roughness = 0.0;
    pbr_input.material.reflectance          = vec3<f32>(0.0);
    pbr_input.frag_coord                    = info.clip_position;
    pbr_input.world_position                = world_position;
    pbr_input.world_normal                  = info.world_coordinate.normal;
    pbr_input.N                             = normalize(info.world_coordinate.normal - surface_gradient);
    pbr_input.V                             = calculate_view(world_position, pbr_input.is_orthographic);

    let lit_color = apply_pbr_lighting(pbr_input).xyz;

   
    return mix(
        sample_attachment(tile, nighttime_attachment, attachments.nighttime).xyz,
        sample_attachment(tile, daytime_attachment, attachments.daytime).xyz,
        1.0 - grayscale(lit_color)
    ) * relief_shading(info.world_coordinate, surface_gradient);
}

fn ocean(tile: AtlasTile, info: FragmentInfo, surface_gradient: vec3<f32>) -> vec3<f32> {
    let world_position = vec4<f32>(apply_height(info.world_coordinate, info.height), 1.0);
    let depth = sample_attachment(tile, bathyometry_attachment, attachments.bathyometry).x;
    let base_ocean_color = calculate_ocean_color(depth);
    let normal = calculate_water_normal(world_position.xz * 0.0000001, info.world_coordinate.normal, globals.time * 10.0, depth);
    var pbr_input: PbrInput                 = pbr_input_new();
    pbr_input.material.base_color           = vec4(base_ocean_color, 1.0);
    pbr_input.material.perceptual_roughness = 1.0;
    pbr_input.material.emissive = vec4(0.0, 0.0, 1.0, 1.0);
    pbr_input.material.reflectance          = vec3<f32>(0.0);
    pbr_input.frag_coord                    = info.clip_position;
    pbr_input.world_position                = world_position;
    pbr_input.world_normal                  = normal;
    pbr_input.N                             = -normalize(normal - surface_gradient);
    //pbr_input.world_normal                  = -info.world_coordinate.normal;
    //pbr_input.N                             = -normalize(info.world_coordinate.normal - surface_gradient);
    pbr_input.V                             = calculate_view(world_position, pbr_input.is_orthographic);

    return apply_pbr_lighting(pbr_input).xyz;
}

// Calculate ocean base color with depth-based blending and chlorophyll influence
fn calculate_ocean_color(depth: f32) -> vec3<f32> {
    let depth_shallow = smoothstep(0.0, 0.3, -depth);
    let depth_deep = smoothstep(0.3, 0.8, depth);

    // Three-tier color blending based on depth
    let shallow_to_medium = mix(
        material.shallow_water_color.xyz,
        material.medium_water_color.xyz,
        depth_shallow
    );
    return mix(
        material.deep_water_color.xyz,
        shallow_to_medium,
        depth_deep
    );
}

// Simple value noise function
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.x, p.y, p.x) * 0.13);
    p3 += dot(p3, vec3(p3.y, p3.z, p3.x) + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let a = hash(i);
    let b = hash(i + vec2(1.0, 0.0));
    let c = hash(i + vec2(0.0, 1.0));
    let d = hash(i + vec2(1.0, 1.0));

    let u = f * f * (3.0 - 2.0 * f);

    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

// Calculate animated water normals with dual-layer scrolling
fn calculate_water_normal(uv: vec2<f32>, world_normal: vec3<f32>, time: f32, depth: f32) -> vec3<f32> {
    let wave_speed1 = 0.03;
    let wave_speed2 = 0.02;
    let wave_dir1 = vec2(1.0, 0.5);
    let wave_dir2 = vec2(-0.5, 1.0);

    // Two layers of scrolling normals for realistic water movement
    let uv1 = uv * 50.0 + wave_dir1 * time * wave_speed1;
    let uv2 = uv * 50.0 + wave_dir2 * time * wave_speed2;

    let normal1 = textureSample(water_normal_texture, water_normal_sampler, uv1).xyz;
    let normal2 = textureSample(water_normal_texture, water_normal_sampler, uv2).xyz;

    // Convert normals from [0,1] to [-1,1] range
    let normal1_unpacked = normal1 * 2.0 - 1.0;
    let normal2_unpacked = normal2 * 2.0 - 1.0;

    // Blend the two normal layers
    let blended_normal = normalize(normal1_unpacked + normal2_unpacked);

    // Depth-varying normal strength: shallow water has more visible wave disturbance
    let depth_deep = smoothstep(0.3, 0.8, depth);
    let normal_strength_shallow = 0.2;  // Choppier waves near shore
    let normal_strength_deep = 0.05;    // Calmer in deep ocean
    let normal_strength = mix(normal_strength_shallow, normal_strength_deep, depth_deep);

    let normalized_world_normal = normalize(world_normal);
    return normalize(normalized_world_normal + blended_normal * normal_strength);
}

@fragment
fn fragment(input: FragmentInput) -> FragmentOutput {
    var info = fragment_info(input);

    let tile = lookup_tile(info.coordinate, info.blend);
    let height_mask = sample_height_mask(tile);

    if height_mask { discard; }

    let surface_gradient = sample_surface_gradient(tile, info.tangent_space);

    let ocean_mask = sample_attachment(tile, ocean_attachment, attachments.ocean).x;

    //var color: vec4<f32> = vec4(1.0);
    var output: FragmentOutput;
    if ocean_mask > 1.0 {
        //output.color = textureSample(water_normal_texture, water_normal_sampler, info.world_coordinate.position.xz * 0.0000001);
        output.color = vec4(ocean(tile, info, surface_gradient), 1.0);
    } else {
        output.color = vec4(terrain(tile, info), 1.0);
    }
    fragment_debug(&info, &output, tile, surface_gradient);
    return output;
}
