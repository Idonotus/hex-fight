use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub type PaletteReference = usize;
pub type RGBA = [u8; 4];

pub trait PaletteAtlas {
    fn add_palette(&mut self, image: &mut Image, palette: Vec<RGBA>) -> PaletteReference {
        let data: Vec<u8> = palette.into_iter().flatten().collect();
        return self.push_data(image, data);
    }
    fn push_data(&mut self, image: &mut Image, data: Vec<u8>) -> PaletteReference;
    fn new_reference(&mut self) -> PaletteReference;
    fn get_image(&self) -> Handle<Image>;
    fn get_size(&self) -> PaletteReference;
}

pub struct BasePalette {
    img: Handle<Image>,
    allocated: usize,
    size: usize,
}

impl BasePalette {
    pub fn new(img: Handle<Image>, size: usize) -> Self {
        Self {
            allocated: 0usize,
            img,
            size,
        }
    }
    pub fn gen_image(size: usize) -> Image {
        let mut i = Image::new(
            Extent3d {
                width: size as u32,
                ..Default::default()
            },
            TextureDimension::D1,
            vec![0; size * 4],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        i.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            min_filter: ImageFilterMode::Linear,
            mag_filter: ImageFilterMode::Nearest,
            ..default()
        });
        return i;
    }
}

impl PaletteAtlas for BasePalette {
    fn push_data(&mut self, image: &mut Image, data: Vec<u8>) -> PaletteReference {
        let image_data: &mut Vec<u8> = image.data.as_mut().unwrap();
        let initial_ref = self.allocated;
        let size: usize = data.len();

        self.allocated += size;

        image_data.splice(initial_ref..self.allocated, data);
        return self.allocated as PaletteReference / 4;
    }

    fn get_image(&self) -> Handle<Image> {
        self.img.clone()
    }

    fn new_reference(&mut self) -> PaletteReference {
        return self.allocated as PaletteReference;
    }

    fn get_size(&self) -> PaletteReference {
        return self.size;
    }
}
