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

	if pixel.r == base.r {
		return vec4<f32>(0.0,0.0,0.0,1.0);
	} else {
		return vec4<f32>(pixel.r*256.0,0.0,0.0,1.0);
	}
	// var coord = vec2<f32>(0.0, (pixel[0] * 256));
	// return vec4f(coord[0], coord[1], pixel[1], 1.0);
}
