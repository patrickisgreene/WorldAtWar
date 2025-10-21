#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_view_bindings::{globals, lights},
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
    road_sampler,
    road_texture,
    night_base_color_texture,
    night_base_color_sampler
}

#import "shaders/earth/bindings.wgsl"::light_sampler;

fn land(in: VertexOutput, is_front: bool) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
   var pbr_input = pbr_input_from_standard_material(in, is_front);
   pbr_input.material.perceptual_roughness = 1.0;

    // Get the directional light direction (assumes first light is the sun)
    let light_dir = lights.directional_lights[0].direction_to_light;
    let world_normal = normalize(in.world_normal);
    let n_dot_l = dot(world_normal, light_dir);

    // Calculate darkness factor (nighttime areas)
    // smoothstep creates smooth transition from day to night
    let darkness = smoothstep(0.1, -0.1, n_dot_l);
    let night_color = textureSample(night_base_color_texture, night_base_color_sampler, in.uv).xyz;
    let earth_color = vec4(mix(pbr_input.material.base_color.xyz, night_color, darkness), 1.0);
    // Sample textures
    //let light_color = vec4(vec3(0.05, 0.04, 0.03), 1.0);
    let road_value = textureSample(road_texture, road_sampler, in.uv);
    pbr_input.material.base_color = vec4(mix(earth_color.xyz, night_color * 8.0, darkness), 1.0);
    pbr_input.material.base_color = mix(pbr_input.material.base_color, vec4(0.0), road_value);
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
#endif
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}