use bevy::{
	prelude::*,
	window::WindowResized
};

#[derive(Component)]
pub struct Anchors {
	pub poses: Rect,
	pub reference: Rect
}

pub struct ResizePlugin;

fn resize_check(mut resize_reader: MessageReader<WindowResized>, elements: Query<&mut Sprite, With<Anchors>>) {
	let Some(last_msg) = resize_reader.read().last() else {return;};

}

impl Plugin for ResizePlugin {
    fn build(&self, app: &mut App) {
		app.add_systems(Update, resize_check);
	}
}

struct Resize(Vec2);
