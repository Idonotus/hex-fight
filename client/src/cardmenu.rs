use bevy::{
    asset::VisitAssetDependencies,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, SystemState},
        world::World,
    },
    input::ButtonState,
    math::{Vec2, Vec3, Vec3Swizzles, primitives::Rectangle, vec2, vec3},
    prelude::*,
    transform::components::{GlobalTransform, Transform},
};
use std::{cmp::min, ops::Add};

use crate::mouse::{ClickBox, ClickEvent, FollowMouse};

pub mod origins;
pub mod selection;

pub trait CardLayout: Sync + Send {
    fn top_left(&self, amount: usize) -> Vec3;
    fn get_bounds(&self, amount: usize) -> Vec3;
    fn grid(&self, amount: usize) -> CardGrid;
    fn on_grid(&self, idx: usize) -> (usize, usize);
    fn position_cards<'a>(&self, amount: usize) -> Box<dyn Iterator<Item = Vec3> + 'a>;
    fn card_size(&self) -> Vec3;
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
            y: gridy * self.layout.card_size.y,
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
    fn grid(&self, amount: usize) -> CardGrid {
        let x = min(self.width, amount);
        let y = (amount / self.width + 1);
        let mut grid = Vec::new();
        for _ in 0..x {
            grid.push(vec![None; y]);
        }
        CardGrid {
            widths: vec![self.card_size.x; x],
            heights: vec![self.card_size.y; y],
            grid,
            card_size: self.card_size,
        }
    }
    fn on_grid(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }
    fn card_size(&self) -> Vec3 {
        self.card_size
    }
}

static REFLECT_Y: Vec3 = Vec3 {
    x: 1.0,
    y: -1.0,
    z: 0.0,
};

pub fn display_groups(mut commands: Commands, cards: Vec<(Entity, CardGroup)>, buffer: f32) {
    let mut currenty = 0.0;
    for (e, group) in cards.iter() {
        let amount = group.cards.len();
        let bounds = group.layout.get_bounds(amount);
        let card_center_offset = group.layout.card_size().with_z(0.0) / 2.0;

        let mut positioner = group.layout.position_cards(amount);
        let mut grid = group.layout.grid(amount);
        for (idx, c) in group.cards.iter().enumerate() {
            let position = positioner
                .next()
                .expect("Positioner ran out of positions when given the expected amount");
            commands.entity(*c).insert(Transform::from_translation(
                (position + card_center_offset) * REFLECT_Y,
            ));
            let (x, y) = group.layout.on_grid(idx);
            grid.grid[x][y] = Some(*c);
        }

        let tleft = group.layout.top_left(amount);

        let centerref =
            -tleft - vec3(bounds.x / 2.0, -currenty, 0.0) - card_center_offset.zyz() * REFLECT_Y;

        commands
            .entity(*e)
            .insert((
                Transform::from_translation(centerref),
                grid,
                ClickBox {
                    bounds: Rectangle::from_size(bounds.xy()),
                    active: true,
                    on_click: menu_click,
                },
            ))
            .add_children(&group.cards);

        currenty -= bounds.y + buffer
    }
}

#[derive(Component)]
struct CardGrid {
    widths: Vec<f32>,
    heights: Vec<f32>,
    grid: Vec<Vec<Option<Entity>>>,
    card_size: Vec3,
}

fn find_class_belonging_to_item<T: Add<Output = T> + PartialOrd + Copy>(
    root: T,
    sizes: &Vec<T>,
    reference: T,
) -> Option<usize> {
    if reference < root {
        return None;
    }
    let mut cumulative = root;

    for (idx, w) in sizes.iter().enumerate() {
        cumulative = cumulative + *w;
        if reference < cumulative {
            return Some(idx);
        }
    }
    return None;
}

impl CardGrid {
    fn remove_at_position(&mut self, position: (usize, usize)) -> Option<Entity> {
        self.grid[position.0][position.1].take()
    }

    fn map_grid_to_local(&self, position: (usize, usize)) -> Vec2 {
        Vec2 {
            x: self.widths[0..position.0].iter().sum(),
            y: self.heights[0..position.1].iter().sum(),
        }
    }

    fn map_world_to_grid(&self, position: Vec2) -> Option<(usize, usize)> {
        let x = find_class_belonging_to_item::<f32>(0.0, &self.widths, position.x);
        if let None = x {
            return None;
        }
        let y = find_class_belonging_to_item::<f32>(0.0, &self.heights, position.y);
        if let None = y {
            return None;
        }
        return Some((x.unwrap(), y.unwrap()));
    }

    fn get_size(&self) -> Vec2 {
        vec2(self.widths.iter().sum(), self.heights.iter().sum())
    }

    fn new(
        widths: Vec<f32>,
        heights: Vec<f32>,
        grid: Vec<Vec<Option<Entity>>>,
        card_size: Vec3,
    ) -> Self {
        Self {
            widths,
            heights,
            grid,
            card_size,
        }
    }

    fn get(&self, position: (usize, usize)) -> Option<Entity> {
        self.grid[position.0][position.1]
    }
}

fn menu_click(world: &mut World, entity: Entity, click: ClickEvent) {
    if click.state != ButtonState::Pressed || click.button != MouseButton::Left {
        return;
    }

    let grid = world.get::<CardGrid>(entity).unwrap();
    let t = match world.get::<GlobalTransform>(entity) {
        Some(t) => t.translation().xy(),
        None => {
            println!("no global trans");
            Vec2::splat(0.0)
        }
    };

    let Some(grid_coord) = grid.map_world_to_grid((click.position - t).reflect(vec2(0.0, 1.0)))
    else {
        return;
    };
    let tleft_box = grid.map_grid_to_local(grid_coord);
    let card_size = grid.card_size;
    let Some(card) = grid.get(grid_coord) else {
        return;
    };

    type S<'w, 's, 'a> = (
        ResMut<'w, selection::Selections>,
        Commands<'w, 's>,
        Query<'w, 's, &'a ChildOf>,
    );

    let g = world.get::<GlobalTransform>(card).unwrap();
    let gtransoffset = g.translation();

    let mut j: SystemState<S> = SystemState::new(world);
    let (mut s, mut c, parents): S = j.get_mut(world);
    // s.0 += 1;
    // let followcount = s.0;
    match s.find(card) {
        Some(idx) => {
            s.remove(idx);
            for (i, e) in s.get_held_cards().iter().enumerate() {
                if i < idx {
                    continue;
                }
                set_follow_index(&mut c, *e, i + 1, click.position);
            }
            c.entity(entity).add_child(card);

            c.entity(card)
                .remove::<FollowMouse>()
                .insert(Transform::from_translation(Vec3 {
                    x: (tleft_box.x + card_size.x / 2.0),
                    y: -(tleft_box.y + card_size.y / 2.0),
                    z: grid_coord.0 as f32 * card_size.z,
                }));
        }
        None => {
            if s.is_full() {
                return;
            }
            s.push(card);
            c.entity(card)
                .insert(Transform::from_translation(gtransoffset));
            if let Ok(parent_group) = parents.get(card) {
                c.entity(parent_group.0).detach_child(card);
            }
            set_follow_index(&mut c, card, s.get_held_cards().len(), click.position);
        }
    }
    j.apply(world);
}

fn set_follow_index(commands: &mut Commands, card: Entity, idx: usize, mouse_pos: Vec2) {
    let offset = vec2(-40.0, 0.0) * idx as f32;
    commands.entity(card).insert((
        FollowMouse::new(offset, idx),
        Transform::from_translation((mouse_pos + offset).xyx().with_z(idx as f32 * 2.0 + 100.0)),
    ));
}

pub fn create_menu_resources(mut commands: Commands) {
    commands.insert_resource(selection::Selections::new(None));
    commands.insert_resource(origins::CardUIOrigins::new());
}
