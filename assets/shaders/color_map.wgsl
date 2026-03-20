#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var base_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var base_color_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(2) var palette_texture: texture_1d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var palette_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> offset: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> cap: f32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
	var pixel: vec4f = textureSample(base_color_texture, base_color_sampler, mesh.uv);
	var base: vec4f = textureSample(base_color_texture, base_color_sampler, vec2f(0.0, 0.0));
	if pixel.a < 0.5 {
		return vec4<f32>(0.0,0.0,0.0,0.0);
	}

	if pixel.r == 1/255 {
		return textureSample(palette_texture, palette_sampler, (offset + 0.5)/cap);
	} else {
		return textureSample(palette_texture, palette_sampler, (offset + 1.5)/cap);
	}
	// return textureSample(palette_texture, palette_sampler, (pixel.r * 100 + offset + 0.5)/cap);
}
