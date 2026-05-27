use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use manaengine::rendering::assetinterface::{PaletteAllocator, PaletteReference};

pub struct BasePalette {
    pub img: Handle<Image>,
    pub(super) allocator: PaletteReservations,
}

impl BasePalette {
    pub fn new(img: Handle<Image>, size: UVec2) -> Self {
        Self {
            allocator: PaletteReservations::new(size),
            img,
        }
    }
    pub fn gen_image(dimensions: UVec2) -> Image {
        return Image::new(
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
    }
}

#[derive(Clone, Copy)]
pub struct PaletteReservations {
    pub allocated: UVec2,
    pub size: UVec2,
}

impl PaletteReservations {
    fn new(size: UVec2) -> Self {
        Self {
            size,
            allocated: UVec2::splat(0),
        }
    }
}

impl PaletteAllocator for PaletteReservations {
    fn allocate(&mut self, size: usize) -> usize {
        if size > self.size.x as usize {
            panic!("Very large palette")
        }
        if size > (self.size.x - self.allocated.x) as usize {
            self.allocated.y += 1;
            self.allocated.x = 0;
        }

        let start = (self.allocated.x + self.allocated.y * self.size.x) as usize;
        self.allocated.x += size as u32;
        return start * 4;
    }

    fn get_size(&self) -> PaletteReference {
        return self.size;
    }

    fn get_ref_from_idx(&self, mut idx: usize) -> PaletteReference {
        idx /= 4;
        let (x, y) = (idx as u32 % self.size.x, idx as u32 / self.size.x);
        return UVec2 { x, y };
    }
}
