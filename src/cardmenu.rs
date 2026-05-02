use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, SystemState, command},
        world::World,
    },
    input::ButtonState,
    math::{Vec2, Vec3, Vec3Swizzles, primitives::Rectangle, vec2, vec3},
    prelude::*,
    transform::components::{GlobalTransform, Transform},
};
use std::{cmp::min, ops::Add};

use crate::mouse::{ClickBox, ClickEvent, DelayQueue, FollowMouse};

mod origins;

pub trait CardLayout: Sync + Send {
    fn top_left(&self, amount: usize) -> Vec3;
    fn get_bounds(&self, amount: usize) -> Vec3;
    fn grid(&self, amount: usize) -> CardGrid;
    fn on_grid(&self, idx: usize) -> (usize, usize);
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
        }
    }
    fn on_grid(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }
}

pub fn display_groups(mut commands: Commands, cards: Vec<(Entity, CardGroup)>, buffer: f32) {
    let mut currenty = 0.0;
    for (e, group) in cards.iter() {
        let amount = group.cards.len();
        let bounds = group.layout.get_bounds(amount);

        let mut positioner = group.layout.position_cards(amount);
        let mut grid = group.layout.grid(amount);
        for (idx, c) in group.cards.iter().enumerate() {
            let position = positioner
                .next()
                .expect("Positioner ran out of positions when given the expected amount");
            commands
                .entity(*c)
                .insert(Transform::from_translation(position));
            let (x, y) = group.layout.on_grid(idx);
            grid.grid[x][y] = Some(*c);
        }

        let tleft = group.layout.top_left(amount);

        let centerref = -tleft - vec3(bounds.x / 2.0, -currenty, 0.0);

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

    fn map_world_to_grid_coord(&self, position: Vec2) -> Option<(usize, usize)> {
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

    fn new(widths: Vec<f32>, heights: Vec<f32>, grid: Vec<Vec<Option<Entity>>>) -> Self {
        Self {
            widths,
            heights,
            grid,
        }
    }

    fn get(&self, position: (usize, usize)) -> Option<Entity> {
        self.grid[position.0][position.1]
    }
}

#[derive(Resource, Default)]
pub struct Selector(usize);

fn menu_click(world: &mut World, entity: Entity, click: ClickEvent) {
    if click.state != ButtonState::Pressed || click.button != MouseButton::Left {
        return;
    }
    dbg!(click);
    let grid = world.get::<CardGrid>(entity).unwrap();
    let t = match world.get::<GlobalTransform>(entity) {
        Some(t) => t.translation().xy(),
        None => {
            println!("no global trans");
            Vec2::splat(0.0)
        }
    };

    let Some(grid_coord) =
        grid.map_world_to_grid_coord((click.position - t).reflect(vec2(0.0, 1.0)))
    else {
        return;
    };
    let Some(card) = grid.get(grid_coord) else {
        return;
    };

    dbg!(card);

    type S<'w, 's, 'a> = (
        ResMut<'w, Selector>,
        Commands<'w, 's>,
        Query<'w, 's, &'a ChildOf>,
    );

    let g = world.get::<GlobalTransform>(card).unwrap();
    let gtransoffset = g.translation();

    let mut j: SystemState<S> = SystemState::new(world);
    let (mut s, mut c, parents): S = j.get_mut(world);
    s.0 += 1;
    let followcount = s.0;
    c.entity(card).insert((
        FollowMouse::new(vec2(-40.0, 0.0) * followcount as f32, followcount),
        Transform::from_translation(gtransoffset.with_z(followcount as f32)),
    ));
    if let Ok(parent_group) = parents.get(card) {
        c.entity(parent_group.0).detach_child(card);
    }
    j.apply(world);
}
