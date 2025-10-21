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
#import "shaders/earth/land.wgsl"::land
#import "shaders/earth/ocean.wgsl"::{lake, ocean}

#import "shaders/earth/bindings.wgsl"::{
    ocean_texture,
    ocean_sampler,
    lake_texture,
    lake_sampler
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let is_ocean = textureSample(ocean_texture, ocean_sampler, in.uv).r;
    let is_lake = textureSample(lake_texture, lake_sampler, in.uv).r;

    if is_ocean == 1.0 {
        return ocean(in, is_front);
    } else if is_lake == 1.0 {
        return lake(in, is_front);
    } else {
        return land(in, is_front);
    }
}