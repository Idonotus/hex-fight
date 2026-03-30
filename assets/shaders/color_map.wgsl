#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var base_color_texture: texture_2d<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> texture_size: vec2<u32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2) var palette_texture: texture_1d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> offset: u32;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> cap: u32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
	let x = clamp(i32(floor(mesh.uv.x * f32(texture_size.x))), 0, i32(texture_size.x) - 1);
	let y = clamp(i32(floor(mesh.uv.y * f32(texture_size.y))), 0, i32(texture_size.y) - 1);
	var pixel: vec4<u32> = textureLoad(base_color_texture, vec2<i32>(x, y), 0);
	if pixel.a == 0 {
		return vec4<f32>(0.0,0.0,0.0,0.0);
	}

	return textureLoad(palette_texture, i32(pixel.r + offset), 0);
}
