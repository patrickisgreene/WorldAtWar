#import "shaders/earth/types.wgsl"::EarthExtension

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var ocean_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var ocean_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var distance_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var distance_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var water_normal_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var water_normal_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(106) var depth_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(107) var depth_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(108) var lake_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(109) var lake_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(110) var chloro_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(111) var chloro_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(112) var road_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(113) var road_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(114) var night_base_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(115) var night_base_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(116)
var<uniform> earth_extension: EarthExtension;