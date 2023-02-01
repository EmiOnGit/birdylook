#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::utils

@group(1) @binding(0)
var<uniform> color: vec4<f32>;
@group(1) @binding(1)
var<uniform> wave_height: f32;
@group(1) @binding(2)
var<uniform> wave_direction: vec2<f32>;
@group(1) @binding(3)
var reflection_texture: texture_2d<f32>;
@group(1) @binding(4)
var base_texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) y: f32,
    @location(1) normal: vec3<f32>,
};
struct Wave {
    length: f32,
    height: f32,
    speed: f32,
    direction: vec2<f32>,
};
// Returns matrix to rotate a vector around a given angle.
// TODO should be const eval since only used for 1 angle.
fn rotation_matrix(angle: f32) -> mat2x2<f32> {
    return mat2x2<f32>(vec2<f32>(cos(angle),-sin(angle)), vec2<f32>(sin(angle), cos(angle)));
}

fn apply_wave(vertex_position: vec2<f32>, wave: Wave) -> f32 {
    let frequency = 2. / wave.length;

    let phase_constant = wave.speed * frequency;
    let delta = sin(dot(wave.direction, vertex_position) * frequency + globals.time * phase_constant);
    return (wave.height * delta);
}
// As described in https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-1-effective-water-simulation-physical-models
fn get_normal(vertex_position: vec2<f32>, wave: Wave) -> vec3<f32> {
    let frequency = 2./wave.length;
    let phase_constant = wave.speed * frequency;
    let normal_x = (frequency * wave.direction.x * wave.height * cos(dot(wave.direction, vertex_position) * frequency + (globals.time * phase_constant)));
    let normal_y = (frequency * wave.direction.y * wave.height * cos(dot(wave.direction, vertex_position) * frequency + (globals.time * phase_constant)));
    return(normalize(vec3<f32>(-normal_x, -normal_y, 1.)));
}
@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>
) -> VertexOutput {

    let rotation_angle = 45.;
    let rotation_matrix1 = rotation_matrix(rotation_angle / 180. * 3.14);

    // init waves
    let wave1 = Wave(0.10, 0.008 * wave_height, 0.05, wave_direction);
    let wave2 = Wave(0.07, 0.004 * wave_height, 0.06, rotation_matrix1 * wave_direction);
    let wave3 = Wave(0.123, 0.006 * wave_height, 0.02, transpose(rotation_matrix1) * wave_direction);
    let wave4 = Wave(0.043, 0.004 * wave_height, 0.03, - wave_direction);

    // extract xy position from vertex
    let vertex_xy = vec2<f32>(vertex_position[0], vertex_position[2]);
    // apply waves to the y offset
    var y_offset = apply_wave(vertex_xy, wave1);
    y_offset += apply_wave(vertex_xy, wave2);
    y_offset += apply_wave(vertex_xy, wave3);
    y_offset += apply_wave(vertex_xy, wave4);
   
    // apply offset to vertex position
    var normal = get_normal(vertex_xy, wave1);
    normal += get_normal(vertex_xy, wave2);
    normal += get_normal(vertex_xy, wave3);
    normal += get_normal(vertex_xy, wave4);
    
    let offset_vector = vec4<f32>(0., y_offset, 0., 0.);

    // create output
    var out: VertexOutput;
    out.normal = normal;
    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex_position, 1.) + offset_vector);
    out.y = y_offset;
    return out;
}

@fragment
fn fragment( 
    in: VertexOutput
) -> @location(0) vec4<f32> {

    var scale_g = 1.;
    var scale_r = 1.;
    var scale_b = 1.;
    if(in.y < -0.002) {
        scale_r = 2.8;
        scale_g = 2.8;
        scale_b = 2.0;
    }
    // We distort the uv with the normal. This should not be the right way and doesn't return good looking distorsions
    var uv = vec2<f32>(0.,1.) + (vec2<f32>(1.,-1.) * coords_to_viewport_uv(in.clip_position.xy, view.viewport) + in.normal.xy * 0.1 );
    let color_pixel =  vec4<f32>(scale_r, scale_g, scale_b, 1.) * color;

    var texture_pixel = textureSample(reflection_texture, base_texture_sampler, uv);

    // Weight of the color defined by the material in contrast to the reflection
    let color_weight =  0.8;
    let texture_weight = 1. - color_weight;
    var resulting_color = color_weight * color_pixel + texture_weight * texture_pixel;

    return(resulting_color);

}
