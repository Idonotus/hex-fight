use std::cmp::min;

use bevy::{
    ecs::{entity::Entity, system::Commands},
    math::{Vec2, Vec3, vec3},
    transform::components::Transform,
};

// #[derive(PartialEq)]
// enum CardGridDimensions {
//     Width(u8),
//     Height(u8),
//     Box(u8, u8),
//     Unbounded,
// }

// pub struct GridPositioner {
//     dimensions: CardGridDimensions,
//     center: Vec2,
//     row: u16,
//     column: u16,
// }

// impl GridPositioner {
//     fn new(dimensions: CardGridDimensions) -> Self {
//         if dimensions == CardGridDimensions::Unbounded {
//             panic!("Can't")
//         }
//         Self { dimensions, center: () }
//     }
// }

pub fn position_cards(
    mut commands: Commands,
    cards: Vec<Entity>,
    width: usize,
    center: Vec3,
    card_size: Vec3,
) {
    let ref_point = card_size.x * min(cards.len(), width) as f32 / 2.0;
    for (idx, c) in cards.iter().enumerate() {
        let (gridx, gridy) = ((idx % width) as f32, (idx / width) as f32);
        commands.entity(*c).insert(Transform::from_translation(
            Vec3 {
                x: gridx * card_size.x - ref_point,
                y: -gridy * card_size.y,
                z: gridx * card_size.z,
            } + center,
        ));
    }
}
