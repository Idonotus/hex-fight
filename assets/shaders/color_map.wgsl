#import bevy_sprite::mesh2d_vertex_output::VertexOutput
// we can import items from shader modules in the assets folder with a quoted path
#import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color_map: array<vec4f>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var base_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var base_color_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
	var pixel: vec4f = textureSample(base_color_texture, base_color_sampler, mesh.uv)[0];
	if pixel[4] < 0.5 {
		return pixel
	}
    var r: i8 = bitcast(pixel*256);
	return color_map[pixel];
}
