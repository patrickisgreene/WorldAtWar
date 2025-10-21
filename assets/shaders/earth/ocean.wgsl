#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_view_bindings::globals,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

#import "shaders/earth/bindings.wgsl"::{
    distance_texture,
    distance_sampler,
    depth_texture,
    depth_sampler,
    chloro_texture,
    chloro_sampler,
    water_normal_texture,
    water_normal_sampler,
    earth_extension
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

// Calculate ocean base color with depth-based blending and chlorophyll influence
fn calculate_ocean_color_chloro(depth: f32, chloro: f32) -> vec3<f32> {
    let depth_color = calculate_ocean_color(depth);
    return mix(depth_color, earth_extension.algea_color.xyz, chloro);
}

// Calculate ocean base color with depth-based blending and chlorophyll influence
fn calculate_ocean_color(depth: f32) -> vec3<f32> {
    let depth_shallow = smoothstep(0.0, 0.3, depth);
    let depth_deep = smoothstep(0.3, 0.8, depth);

    // Three-tier color blending based on depth
    let shallow_to_medium = mix(
        earth_extension.shallow_water_color.xyz,
        earth_extension.medium_water_color.xyz,
        depth_shallow
    );
    return mix(
        shallow_to_medium,
        earth_extension.deep_water_color.xyz,
        depth_deep
    );
}

// Calculate shore ripples and foam effect
fn calculate_shore_ripples(distance: f32, uv: vec2<f32>, time: f32) -> f32 {
    let noise_scale = 0.1;
    let noise_strength = 3.0;
    let noise_offset = value_noise(uv * noise_scale) * noise_strength;
    let wave = sin(((distance + noise_offset) * earth_extension.ripple_frequency) - (time * earth_extension.ripple_speed)) * 0.5 + 0.5;
    let ripple_fade = smoothstep(0.0, earth_extension.ripple_distance, distance);

    return wave * (1.0 - ripple_fade);
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

fn ocean(in: VertexOutput, is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let distance = textureSample(distance_texture, distance_sampler, in.uv).r;
    let depth = 1.0 - textureSample(depth_texture, depth_sampler, in.uv).r;
    let time = globals.time;

    // Calculate ocean color with depth and chlorophyll influence
    let chloro = textureSample(chloro_texture, chloro_sampler, in.uv).r;
    let base_ocean_color = calculate_ocean_color_chloro(depth, chloro);

    // Calculate shore ripples and combine with base color
    let ripple_intensity = calculate_shore_ripples(distance, in.uv, time);
    let shore_foam = vec3(0.85, 0.92, 0.96);
    let final_ocean_color = mix(base_ocean_color, shore_foam, ripple_intensity);

    // Set PBR material properties
    pbr_input.material.base_color = vec4(final_ocean_color, 1.0);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // Depth-varying roughness
    let depth_deep = smoothstep(0.3, 0.8, depth);
    let roughness_shallow = 0.25;
    let roughness_deep = 0.05;
    pbr_input.material.perceptual_roughness = mix(roughness_shallow, roughness_deep, depth_deep);

    pbr_input.material.metallic = 0.0;
    pbr_input.material.reflectance = vec3(0.5);

    // Calculate animated water normals
    pbr_input.N = calculate_water_normal(in.uv, in.world_normal, time, depth);
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}

fn lake(in: VertexOutput, is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let distance = textureSample(distance_texture, distance_sampler, in.uv).r;
    let depth = 1.0 - textureSample(depth_texture, depth_sampler, in.uv).r;
    let time = globals.time;

    // Calculate ocean color with depth and chlorophyll influence
    let base_ocean_color = calculate_ocean_color(depth);

    // Calculate shore ripples and combine with base color
    let ripple_intensity = calculate_shore_ripples(distance, in.uv, time);
    let shore_foam = vec3(0.85, 0.92, 0.96);
    let final_ocean_color = mix(base_ocean_color, shore_foam, ripple_intensity);

    // Set PBR material properties
    pbr_input.material.base_color = vec4(base_ocean_color, 1.0);
    //pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // Depth-varying roughness
    let depth_deep = smoothstep(0.3, 0.8, depth);
    let roughness_shallow = 0.25;
    let roughness_deep = 0.05;
    pbr_input.material.perceptual_roughness = mix(roughness_shallow, roughness_deep, depth_deep);

    pbr_input.material.metallic = 0.0;
    pbr_input.material.reflectance = vec3(0.5);

    // Calculate animated water normals
    pbr_input.N = calculate_water_normal(in.uv, in.world_normal, time, depth);
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}