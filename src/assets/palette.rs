use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub type PaletteReference = UVec2;
pub type RGBA = [u8; 4];

pub trait PaletteAtlas {
    fn add_palette(&mut self, image: &mut Image, palette: &[RGBA]) -> PaletteReference {
        let (start, end) = self.allocate(palette.len());
        let data: Vec<u8> = palette.iter().flatten().copied().collect();
        self.push_data(image, data, start, end);
        return self.get_ref_from_idx(start);
    }
    fn push_data(&mut self, image: &mut Image, data: Vec<u8>, start: usize, end: usize);
    fn allocate(&mut self, size: usize) -> (usize, usize);
    fn get_ref_from_idx(&self, idx: usize) -> PaletteReference;
    fn get_image(&self) -> Handle<Image>;
    fn get_size(&self) -> PaletteReference;
}

pub struct BasePalette {
    img: Handle<Image>,
    allocated: UVec2,
    size: UVec2,
}

impl BasePalette {
    pub fn new(img: Handle<Image>, size: UVec2) -> Self {
        Self {
            allocated: UVec2 { x: 0, y: 0 },
            img,
            size,
        }
    }
    pub fn gen_image(dimensions: UVec2) -> Image {
        let mut i = Image::new(
            Extent3d {
                width: dimensions.x,
                height: dimensions.y,
                ..Default::default()
            },
            TextureDimension::D2,
            vec![255; (dimensions.x * dimensions.y * 4) as usize],
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
    fn push_data(&mut self, image: &mut Image, data: Vec<u8>, start: usize, end: usize) {
        let image_data: &mut Vec<u8> = image.data.as_mut().unwrap();
        image_data.splice(start..end, data);
    }

    fn get_image(&self) -> Handle<Image> {
        self.img.clone()
    }

    fn allocate(&mut self, size: usize) -> (usize, usize) {
        if size > self.size.x as usize {
            panic!("Very large palette")
        }
        if size > (self.size.x - self.allocated.x) as usize {
            self.allocated.y += 1;
            self.allocated.x = 0;
        }

        let start = (self.allocated.x + self.allocated.y * self.size.x) as usize;
        self.allocated.x += size as u32;
        return (start * 4, (start + size) * 4);
    }

    fn get_ref_from_idx(&self, mut idx: usize) -> PaletteReference {
        idx /= 4;
        let (x, y) = (idx as u32 % self.size.x, idx as u32 / self.size.x);
        return UVec2 { x, y };
    }

    fn get_size(&self) -> PaletteReference {
        return self.size;
    }
}
