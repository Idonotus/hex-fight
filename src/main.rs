use bevy::prelude::*;

mod engine;

fn main() {
    App::new()
    .add_plugins(engine::EnginePlugin)
    .run();
}