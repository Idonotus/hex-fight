use std::mem::swap;

use bevy::{
    ecs::system::SystemState,
    input::{ButtonState, mouse::MouseButtonInput},
    platform::collections::HashSet,
    prelude::*,
};

pub struct MousePlugin;

#[derive(Debug, Clone, Copy)]
pub struct ClickEvent {
    pub button: MouseButton,
    pub state: ButtonState,
    pub position: Vec2,
}

impl ClickEvent {
    fn new(bi: MouseButtonInput, pos: Vec2) -> Self {
        Self {
            button: bi.button,
            state: bi.state,
            position: pos,
        }
    }
}

#[derive(Component)]
pub struct ClickBox {
    pub bounds: Rectangle,
    pub active: bool,
    pub on_click: fn(&mut World, Entity, ClickEvent) -> (),
}

struct ClickQueue {
    entity: Entity,
    clicks: Vec<MouseButtonInput>,
    on_click: fn(&mut World, Entity, ClickEvent) -> (),
}

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_clicks.after(update_position),
                follow_mouse.after(update_position),
                update_position,
            ),
        )
        .add_systems(Startup, |world: &mut World| {
            world.insert_resource(ButtonEdge::default());
            world.insert_resource(MouseWorldPosition::default());
        });
    }
}

#[derive(Resource, Default)]
struct ButtonEdge {
    buttons: HashSet<MouseButton>,
}

#[derive(Resource, Default)]
pub struct MouseWorldPosition(pub Vec2);

fn update_position(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MouseWorldPosition>,
) {
    let w: &Window = window.single().unwrap();
    let (c, ctrans): (&Camera, &GlobalTransform) = camera.single().unwrap();
    let Some(mpos) = w
        .cursor_position()
        .and_then(|x| c.viewport_to_world_2d(ctrans, x).ok())
    else {
        return;
    };
    mouse_pos.0 = mpos
}

fn check_clicks(world: &mut World) {
    type S<'w, 's, 'a> = (
        Res<'w, MouseWorldPosition>,
        Query<'w, 's, (&'a mut ClickBox, &'a GlobalTransform, Entity)>,
        ResMut<'w, ButtonEdge>,
        MessageReader<'w, 's, MouseButtonInput>,
    );
    let mut sys: SystemState<S> = SystemState::new(world);
    let (mouse_pos, boxes, mut helds, mut msgs): S = sys.get_mut(world);
    let mpos = mouse_pos.0;

    let mut click_queue: Vec<ClickQueue> = Vec::new();
    let mut clicks: Vec<MouseButtonInput> = Vec::new();

    for click in msgs.read().into_iter() {
        let prev_held = helds.buttons.contains(&click.button);
        if !(prev_held ^ click.state.is_pressed()) {
            continue;
        }

        if prev_held {
            helds.buttons.remove(&click.button);
        } else {
            helds.buttons.insert(click.button);
        }

        clicks.push(click.clone());
    }

    if clicks.len() == 0 {
        return;
    }

    for (b, t, e) in boxes {
        if !b.active {
            continue;
        }
        let mut buttons: Vec<MouseButtonInput> = Vec::new();
        let pos = t.translation().xy();
        let relclick = mpos - pos;

        for click in clicks.iter() {
            if relclick.x < 0.0 || relclick.y > 0.0 {
                continue;
            }

            let size = b.bounds.size();

            if relclick.x > size.x || relclick.y < -size.y {
                continue;
            }

            buttons.push(click.clone())
        }
        click_queue.push(ClickQueue {
            entity: e,
            clicks: buttons,
            on_click: b.on_click,
        });
    }

    for q in click_queue {
        for c in q.clicks {
            (q.on_click)(world, q.entity, ClickEvent::new(c, mpos))
        }
    }
}

pub struct DelayQueue<T> {
    length: usize,
    head: usize,
    pos_queue: Vec<Option<T>>,
}

impl<T> DelayQueue<T> {
    fn new(size: usize) -> Self {
        let mut pos_queue = Vec::with_capacity(size);
        for _ in 0..size {
            pos_queue.push(None);
        }
        Self {
            length: size,
            head: 0,
            pos_queue,
        }
    }

    fn push(&mut self, item: T) -> Option<T> {
        let mut c = Some(item);
        if self.length == 0 {
            return c;
        }
        swap(&mut c, &mut self.pos_queue[self.head]);

        self.head = (self.head + 1) % self.length;

        return c;
    }
}

#[derive(Component)]
pub struct FollowMouse {
    pub offset: Vec2,
    pub pos_queue: DelayQueue<Vec2>,
}

impl FollowMouse {
    pub fn new(offset: Vec2, delay: usize) -> Self {
        Self {
            offset,
            pos_queue: DelayQueue::new(delay),
        }
    }
}

fn follow_mouse(
    mouse_pos: Res<MouseWorldPosition>,
    followers: Query<(&mut Transform, &mut FollowMouse)>,
) {
    for (mut t, mut m) in followers {
        let follow_pos = mouse_pos.0 + m.offset;
        match m.pos_queue.push(follow_pos) {
            Some(p) => {
                t.translation.x = p.x;
                t.translation.y = p.y;
            }
            _ => {
                continue;
            }
        }
    }
}
