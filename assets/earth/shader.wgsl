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
    var pbr_input: PbrInput                 = pbr_input_new();
    pbr_input.material.base_color           = vec4(base_ocean_color, 1.0);
    pbr_input.material.perceptual_roughness = 0.0;
    pbr_input.material.reflectance          = vec3<f32>(0.0);
    pbr_input.frag_coord                    = info.clip_position;
    pbr_input.world_position                = world_position;
    pbr_input.world_normal                  = -info.world_coordinate.normal;
    pbr_input.N                             = -normalize(info.world_coordinate.normal - surface_gradient);
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
        output.color = vec4(ocean(tile, info, surface_gradient), 1.0);
    } else {
        output.color = vec4(terrain(tile, info), 1.0);
    }
    fragment_debug(&info, &output, tile, surface_gradient);
    return output;
}
