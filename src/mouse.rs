use std::mem::swap;

use bevy::{ecs::system::SystemState, input::{ButtonState, mouse::MouseButtonInput}, prelude::*};

pub struct MousePlugin;


pub struct ClickEvent {
    pub button: MouseButton,
    pub state: ButtonState,
    pub position: Vec2,
}

impl ClickEvent {
    fn new(bi: MouseButtonInput, pos: Vec2) -> Self {
        Self { button: bi.button, state: bi.state, position: pos }
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
        app.add_systems(Update, (check_clicks, follow_mouse));
    }
}

fn check_clicks(world: &mut World) {
    type S<'w, 's, 'a> = (
        Query<'w, 's, &'a Window>,
        Query<'w, 's, (&'a Camera, &'a GlobalTransform)>,
        Query<'w, 's, (&'a mut ClickBox, &'a GlobalTransform, Entity)>,
        MessageReader<'w, 's, MouseButtonInput>
    );
    let mut sys: SystemState<S> = SystemState::new(world);
    let (
        windows,
        cams,
        boxes,
        mut msgs): S = sys.get_mut(world);
    let w: &Window = windows.single().unwrap();
    let (c, ctrans): (&Camera, &GlobalTransform) = cams.single().unwrap();
    let mpos = w.cursor_position().and_then(|x| c.viewport_to_world_2d(ctrans, x).ok()).unwrap();

    let mut click_queue: Vec<ClickQueue> = Vec::new();
    let mut clicks: Vec<MouseButtonInput> = Vec::new();

    for click in msgs.read().into_iter() {
        clicks.push(click.clone());
    }

    if clicks.len() == 0 {return;}


    for (b, t, e) in boxes {
        if !b.active {continue;}
        let mut buttons: Vec<MouseButtonInput> = Vec::new();
        for click in clicks.iter() {
            let pos = t.translation().xy();
            let relclick = mpos - pos;

            if relclick.x < 0.0 || relclick.y < 0.0 { continue; }

            let size = b.bounds.size();

            if relclick.x > size.x || relclick.y > size.y { continue; }

            buttons.push(click.clone())
        }
        click_queue.push(ClickQueue { entity: e, clicks: buttons, on_click: b.on_click });
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
    pos_queue: Vec<Option<T>>
}

impl<T> DelayQueue<T> {
    fn new(size: usize) -> Self {
        Self {
            length: size,
            head: 0,
            pos_queue: Vec::with_capacity(size)
        }
    }

    fn push(&mut self, item: T) -> Option<T> {
        let mut c = Some(item);
        if self.length == 0 {
            return c;
        }
        swap(&mut c, &mut self.pos_queue[self.head]);

        self.head = self.head + 1 % self.length;

        return c;
    }
}

#[derive(Component)]
pub struct FollowMouse {
    offset: Vec2,
    pos_queue: DelayQueue<Vec2>
}

fn follow_mouse(wq: Query<&Window>, cq: Query<(&Camera, &GlobalTransform)>, followers: Query<(&mut Transform, &mut FollowMouse)>) {
    let w: &Window = wq.single().unwrap();
    let (c, ctrans): (&Camera, &GlobalTransform) = cq.single().unwrap();
    let mpos = w.cursor_position().and_then(|x| c.viewport_to_world_2d(ctrans, x).ok()).unwrap();
    for (mut t, mut m) in followers {
        let follow_pos = mpos + m.offset;
        match m.pos_queue.push(follow_pos) {
            Some(p) => {
                t.translation.x = p.x;
                t.translation.y = p.y;
            }
            _ => {continue;}
        }
    }
}