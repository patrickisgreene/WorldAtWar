#define_import_path waw_earth_render::functions

#import waw_earth_render::attachments::compute_sample_uv
#import waw_earth_render::types::AttachmentConfig
#import waw_earth_render::bindings::{earth, earth_sampler, origins, earth_view, geometry_tiles, tile_tree, view, approximate_height}
#import waw_earth_render::types::{TileCoordinate, WorldCoordinate, TileTree, TileTreeEntry, AtlasTile, Blend, BestLookup, Coordinate, Morph, TangentSpace}
#import bevy_render::maths::{affine3_to_square, mat2x4_f32_to_mat3x3_unpack}

const SIGMA = 0.87 * 0.87;

fn high_precision(view_distance: f32) -> bool {
#ifdef HIGH_PRECISION
    return view_distance < earth_view.precision_distance;
#else
    return false;
#endif
}

#ifdef VERTEX
fn compute_coordinate(vertex_index: u32) -> Coordinate {
    // use first and last indices of the rows twice, to form degenerate triangles
    let tile_index   = vertex_index / earth_view.vertices_per_tile;
    let column_index = vertex_index % earth_view.vertices_per_tile / earth_view.vertices_per_row;
    let row_index    = clamp(vertex_index % earth_view.vertices_per_row, 1u, earth_view.vertices_per_row - 2u) - 1u;
    let grid_index   = vec2<u32>(column_index + (row_index & 1u), row_index >> 1u);

    let tile    = geometry_tiles[tile_index];
    let tile_uv = vec2<f32>(grid_index) / earth_view.grid_size;
    let even_uv = vec2<f32>(grid_index & vec2<u32>(~1u)) / earth_view.grid_size;

    let morph_ratio = mix(mix(tile.morph_ratios.x, tile.morph_ratios.y, tile_uv.x),
                          mix(tile.morph_ratios.z, tile.morph_ratios.w, tile_uv.x), tile_uv.y);

    return Coordinate(tile.face, tile.lod, tile.xy, mix(tile_uv, even_uv, morph_ratio));
}
#endif

#ifdef FRAGMENT
fn compute_coordinate(tile_index: u32, tile_uv: vec2<f32>) -> Coordinate {
    let tile = geometry_tiles[tile_index];

    return Coordinate(tile.face, tile.lod, tile.xy, tile_uv, dpdx(tile_uv), dpdy(tile_uv));
}
#endif


#ifdef PREPASS
fn compute_world_coordinate(coordinate: Coordinate) -> WorldCoordinate {
    var world_coordinate = compute_world_coordinate_imprecise(coordinate, approximate_height);

    if (high_precision(world_coordinate.view_distance)) {
        world_coordinate = compute_world_coordinate_precise(coordinate, approximate_height);
    }

    return world_coordinate;
}
#endif

#ifdef VERTEX
fn compute_world_coordinate(coordinate: Coordinate, tile_index: u32, tile_uv: vec2<f32>) -> WorldCoordinate {
    let tile          = geometry_tiles[tile_index];
    let view_distance = mix(mix(tile.view_distances.x, tile.view_distances.y, tile_uv.x),
                            mix(tile.view_distances.z, tile.view_distances.w, tile_uv.x), tile_uv.y);

    if (high_precision(view_distance)) { return compute_world_coordinate_precise(coordinate, approximate_height); }
    else {                               return compute_world_coordinate_imprecise(coordinate, approximate_height); }
}
#endif

#ifdef FRAGMENT
fn compute_world_coordinate(coordinate: Coordinate, height: f32, view_distance: f32) -> WorldCoordinate {
    if (high_precision(view_distance)) { return compute_world_coordinate_precise(coordinate, height); }
    else {                               return compute_world_coordinate_imprecise(coordinate, height); }
}

fn sample_attachment(tile: AtlasTile, attachment: texture_2d_array<f32>, attachment_label: AttachmentConfig) -> vec4<f32> {
    let uv = compute_sample_uv(tile, attachment_label);

#ifdef SAMPLE_GRAD
    return textureSampleGrad(attachment, earth_sampler, uv.uv, tile.index, uv.dx, uv.dy);
#else
    return textureSampleLevel(attachment, earth_sampler, uv.uv, tile.index, tile.blend_ratio);
#endif
}
#endif


fn compute_world_coordinate_imprecise(coordinate: Coordinate, height: f32) -> WorldCoordinate {
    let uv = (vec2<f32>(coordinate.xy) + coordinate.uv) / exp2(f32(coordinate.lod));

    let xy = (2.0 * uv - 1.0) / sqrt(1.0 - 4.0 * SIGMA * (uv - 1.0) * uv);

    // this is faster than the CPU SIDE_MATRICES approach
    var unit_position: vec3<f32>;
    switch (coordinate.face) {
        case 0u: { unit_position = vec3( -1.0, -xy.y,  xy.x); }
        case 1u: { unit_position = vec3( xy.x, -xy.y,   1.0); }
        case 2u: { unit_position = vec3( xy.x,   1.0,  xy.y); }
        case 3u: { unit_position = vec3(  1.0, -xy.x,  xy.y); }
        case 4u: { unit_position = vec3( xy.y, -xy.x,  -1.0); }
        case 5u: { unit_position = vec3( xy.y,  -1.0,  xy.x); }
        case default: {}
    }

    unit_position   = normalize(unit_position);
    let unit_normal = unit_position;

    let position_world_from_unit = affine3_to_square(earth.world_from_unit);
    let world_position           = (position_world_from_unit * vec4<f32>(unit_position, 1.0)).xyz;

    let normal_world_from_unit = mat2x4_f32_to_mat3x3_unpack(earth.unit_from_world_transpose_a, earth.unit_from_world_transpose_b);
    let world_normal           = normalize(normal_world_from_unit * unit_normal);

    let view_distance = distance(world_position + height * world_normal, earth_view.world_position);

    return WorldCoordinate(world_position, world_normal, view_distance);
}

#ifdef HIGH_PRECISION
fn compute_world_coordinate_precise(coordinate: Coordinate, height: f32) -> WorldCoordinate {
    let view_coordinate = compute_view_coordinate(coordinate.face, coordinate.lod);

    let relative_uv = (vec2<f32>(vec2<i32>(coordinate.xy) - vec2<i32>(view_coordinate.xy)) + coordinate.uv - view_coordinate.uv) / exp2(f32(coordinate.lod));
    let u = relative_uv.x;
    let v = relative_uv.y;

    let approximation = earth_view.surface_approximation[coordinate.face];

    let world_position = approximation.p + approximation.p_u * u + approximation.p_v * v +
                         approximation.p_uu * u * u + approximation.p_uv * u * v + approximation.p_vv * v * v;
    let world_normal = normalize(cross(approximation.p_v, approximation.p_u)); // normal at viewer coordinate good enough?

    let view_distance = distance(world_position + height * world_normal, earth_view.world_position);

    return WorldCoordinate(world_position, world_normal, view_distance);
}
#else
fn compute_world_coordinate_precise(coordinate: Coordinate, height: f32) -> WorldCoordinate { return WorldCoordinate(vec3<f32>(0.0), vec3<f32>(0.0), 0.0); }
#endif

fn compute_tangent_space(world_coordinate: WorldCoordinate) -> TangentSpace {
    let position_dx = dpdx(world_coordinate.position);
    let position_dy = dpdy(world_coordinate.position);

    // Check if derivatives are valid (not too small, which can happen at extreme zoom levels)
    let dx_length = length(position_dx);
    let dy_length = length(position_dy);
    let min_derivative = 1e-10;

    if (dx_length < min_derivative || dy_length < min_derivative) {
        // Fallback: construct orthonormal basis from normal when derivatives are degenerate
        let up = select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(world_coordinate.normal.y) > 0.999);
        let tangent_x = normalize(cross(up, world_coordinate.normal));
        let tangent_y = cross(world_coordinate.normal, tangent_x);
        return TangentSpace(tangent_x, tangent_y, 1.0);
    }

    // Compute orthonormal tangent vectors
    // tangent_x should be perpendicular to both position_dy and normal
    // tangent_y should be perpendicular to both normal and position_dx
    let tangent_x_raw = cross(position_dy, world_coordinate.normal);
    let tangent_y_raw = cross(world_coordinate.normal, position_dx);

    // Check if cross products are valid
    let tx_length = length(tangent_x_raw);
    let ty_length = length(tangent_y_raw);

    if (tx_length < min_derivative || ty_length < min_derivative) {
        // Fallback if cross products degenerate
        let up = select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(world_coordinate.normal.y) > 0.999);
        let tangent_x = normalize(cross(up, world_coordinate.normal));
        let tangent_y = cross(world_coordinate.normal, tangent_x);
        return TangentSpace(tangent_x, tangent_y, 1.0);
    }

    let tangent_x = tangent_x_raw / tx_length;
    let tangent_y = tangent_y_raw / ty_length;

    // Compute scale factor with safety clamping
    let denominator = dot(position_dx, tangent_x);
    let scale = 1.0 / max(abs(denominator), 1e-7);

    return TangentSpace(tangent_x, tangent_y, scale);
}

fn apply_height(world_coordinate: WorldCoordinate, height: f32) -> vec3<f32> {
    return world_coordinate.position + height * world_coordinate.normal;
}

fn inverse_mix(a: f32, b: f32, value: f32) -> f32 {
    return saturate((value - a) / (b - a));
}

fn compute_morph(lod: u32, view_distance: f32) -> f32 {
#ifdef MORPH
    let target_lod = log2(earth_view.morph_distance / view_distance);

    return select(saturate(1.0 - (target_lod - f32(lod)) / earth_view.morph_range), 0.0, lod == 0);
#else
    return 0.0;
#endif
}

fn compute_blend(view_distance: f32) -> Blend {
    let target_lod = log2(earth_view.blend_distance / view_distance);

#ifdef BLEND
    let ratio = saturate(1.0 - fract(target_lod) / earth_view.blend_range);
#else
    let ratio = 0.0;
#endif

    return Blend(min(u32(target_lod), earth.lod_count - 1), select(ratio, 0.0, target_lod < 1 || u32(target_lod) >= earth.lod_count));
}

fn compute_view_coordinate(face: u32, lod: u32) -> Coordinate {
    let coordinate = earth_view.coordinates[face];

#ifdef FRAGMENT
    var view_coordinate = Coordinate(face, earth_view.lod, coordinate.xy, coordinate.uv, vec2<f32>(0.0), vec2<f32>(0.0));
#else
    var view_coordinate = Coordinate(face, earth_view.lod, coordinate.xy, coordinate.uv);
#endif

    coordinate_change_lod(&view_coordinate, lod);

    return view_coordinate;
}

fn compute_subdivision_coordinate(tile: TileCoordinate) -> Coordinate {
    let view_coordinate = compute_view_coordinate(tile.face, tile.lod);

    var offset = vec2<i32>(view_coordinate.xy) - vec2<i32>(tile.xy);
    var uv     = view_coordinate.uv;

    if      (offset.x < 0) { uv.x = 0.0; }
    else if (offset.x > 0) { uv.x = 1.0; }
    if      (offset.y < 0) { uv.y = 0.0; }
    else if (offset.y > 0) { uv.y = 1.0; }

#ifdef FRAGMENT
    return Coordinate(tile.face, tile.lod, tile.xy, uv, vec2<f32>(0.0), vec2<f32>(0.0));
#else
    return Coordinate(tile.face, tile.lod, tile.xy, uv);
#endif
}

fn coordinate_change_lod(coordinate: ptr<function, Coordinate>, new_lod: u32) {
    let lod_difference = i32(new_lod) - i32((*coordinate).lod);

    if (lod_difference == 0) { return; }

    let scale = exp2(f32(lod_difference));
    let xy = (*coordinate).xy;
    let uv = (*coordinate).uv * scale;

    (*coordinate).lod = new_lod;
    (*coordinate).xy = vec2<u32>(vec2<f32>(xy) * scale) + vec2<u32>(uv);
    (*coordinate).uv = uv % 1.0 + select(vec2<f32>(xy % u32(1 / scale)) * scale, vec2<f32>(0.0), lod_difference > 0);

#ifdef FRAGMENT
    (*coordinate).uv_dx *= scale;
    (*coordinate).uv_dy *= scale;
#endif
}

fn compute_tile_tree_uv(coordinate: Coordinate) -> vec2<f32> {
    let view_coordinate = compute_view_coordinate(coordinate.face, coordinate.lod);

    let tile_count = i32(exp2(f32(coordinate.lod)));
    let tree_size  = min(i32(earth_view.tree_size), tile_count);
    let tree_xy    = vec2<i32>(view_coordinate.xy) + vec2<i32>(round(view_coordinate.uv)) - vec2<i32>(earth_view.tree_size / 2);
    let view_xy    = clamp(tree_xy, vec2<i32>(0), vec2<i32>(tile_count - tree_size));

    return (vec2<f32>(vec2<i32>(coordinate.xy) - view_xy) + coordinate.uv) / f32(tree_size);
}


fn lookup_tile_tree_entry(coordinate: Coordinate) -> TileTreeEntry {
    let tree_xy    = vec2<u32>(coordinate.xy) % earth_view.tree_size;
    let tree_index = ((coordinate.face * earth.lod_count +
                       coordinate.lod) * earth_view.tree_size +
                       tree_xy.x)      * earth_view.tree_size +
                       tree_xy.y;

    return tile_tree[tree_index];
}

// Todo: implement this more efficiently
fn lookup_best(lookup_coordinate: Coordinate) -> BestLookup {
    var coordinate: Coordinate; var tile_tree_uv: vec2<f32>;

    var new_coordinate   = lookup_coordinate;
    coordinate_change_lod(&new_coordinate , 0u);
    var new_tile_tree_uv = new_coordinate.uv;

    while (new_coordinate.lod < earth.lod_count && !any(new_tile_tree_uv <= vec2<f32>(0.0)) && !any(new_tile_tree_uv >= vec2<f32>(1.0))) {
        coordinate  = new_coordinate;
        tile_tree_uv = new_tile_tree_uv;

        new_coordinate = lookup_coordinate;
        coordinate_change_lod(&new_coordinate, coordinate.lod + 1u);
        new_tile_tree_uv = compute_tile_tree_uv(new_coordinate);
    }

    let tile_tree_entry = lookup_tile_tree_entry(coordinate);

    coordinate_change_lod(&coordinate, tile_tree_entry.atlas_lod);

    return BestLookup(AtlasTile(tile_tree_entry.atlas_index, coordinate, 0.0), tile_tree_uv);
}

fn lookup_tile(lookup_coordinate: Coordinate, blend: Blend) -> AtlasTile {
#ifdef TILE_TREE_LOD
    return lookup_best(lookup_coordinate).tile;
#else
    var coordinate = lookup_coordinate;

    coordinate_change_lod(&coordinate, blend.lod);

    let tile_tree_entry = lookup_tile_tree_entry(coordinate);

    coordinate_change_lod(&coordinate, tile_tree_entry.atlas_lod);

    return AtlasTile(tile_tree_entry.atlas_index, coordinate, blend.ratio);
#endif
}
