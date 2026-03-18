#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var base_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var base_color_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(2) var palette_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var palette_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> offset: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> cap: f32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
	var pixel: vec4f = textureSample(base_color_texture, base_color_sampler, mesh.uv);
	if pixel[4] < 0.5 {
		return pixel
	}
    var coord = vec2<f32>((pixel[0] * 256 + offset)/cap, 0.0);
	return textureSample(palette_texture, palette_sampler, coord);
}
