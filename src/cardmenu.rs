use std::cmp::min;

use bevy::{
    ecs::{component::Component, entity::Entity, system::Commands},
    math::{Vec2, Vec3, vec3},
    transform::components::Transform,
};

pub trait CardLayout: Sync + Send {
    fn top_left(&self, amount: usize) -> Vec3;
    fn get_bounds(&self, amount: usize) -> Vec3;
    fn position_cards<'a>(&self, amount: usize) -> Box<dyn Iterator<Item = Vec3> + 'a>;
}

#[derive(Component)]
pub struct CardGroup {
    pub name: String,
    pub layout: Box<dyn CardLayout>,
    pub cards: Vec<Entity>,
}

#[derive(Clone, Copy)]
pub struct WidthLayout {
    pub width: usize,
    pub card_size: Vec3,
}

struct WidthPositioner {
    layout: WidthLayout,
    amount: usize,
    cur: usize,
}

impl Iterator for WidthPositioner {
    type Item = Vec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.amount {
            return None;
        }
        let (gridx, gridy) = (
            (self.cur % self.layout.width) as f32,
            (self.cur / self.layout.width) as f32,
        );
        self.cur += 1;
        return Some(Vec3 {
            x: gridx * self.layout.card_size.x,
            y: gridy * -self.layout.card_size.y,
            z: gridx * self.layout.card_size.z,
        });
    }
}

impl CardLayout for WidthLayout {
    fn top_left(&self, _amount: usize) -> Vec3 {
        Vec3::splat(0.0)
    }
    fn get_bounds(&self, amount: usize) -> Vec3 {
        Vec3 {
            x: min(self.width, amount) as f32 * self.card_size.x,
            y: (amount / self.width + 1) as f32 * self.card_size.y,
            z: min(self.width, amount) as f32 * self.card_size.z,
        }
    }
    fn position_cards<'a>(&self, amount: usize) -> Box<dyn Iterator<Item = Vec3> + 'a> {
        Box::new(WidthPositioner {
            layout: self.clone(),
            amount,
            cur: 0,
        })
    }
}

pub fn display_groups(mut commands: Commands, cards: Vec<(Entity, CardGroup)>, buffer: f32) {
    let mut currenty = 0.0;
    for (e, group) in cards.iter() {
        let amount = group.cards.len();
        let bounds = group.layout.get_bounds(amount);
        let tleft = group.layout.top_left(amount);

        commands
            .entity(*e)
            .insert(Transform::from_xyz(0.0, currenty - tleft.y, 0.0));

        let centerref = -tleft - vec3(bounds.x / 2.0, 0.0, 0.0);
        let mut positioner = group.layout.position_cards(amount);

        for c in group.cards.iter() {
            let position = positioner
                .next()
                .expect("Positioner ran out of positions when given the expected amount");
            commands
                .entity(*c)
                .insert(Transform::from_translation(position + centerref));
        }

        currenty -= bounds.y + buffer
    }
}
