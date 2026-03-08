use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::{AlphaMode2d, Material2d}};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RecolourMaterial {
    #[uniform(0)]
    pub(super) pallete: Vec<LinearRgba>,
    #[texture(1)]
    #[sampler(2)]
    pub(super) color_texture: Option<Handle<Image>>,
}

impl Material2d for RecolourMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/color_map.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Mask(0.5)
    }
}