use bevy::prelude::*;

mod assetmanager;
mod cardmenu;
mod mouse;
mod orchestrator;
mod resizeplugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    file_path: "../assets".to_owned(),

                    ..Default::default()
                }),
            mouse::MousePlugin,
            resizeplugin::ResizePlugin,
            orchestrator::GamePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

// fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn((Sprite {
//         image: asset_server.load("handbar.png"),
//         custom_size: Some(Vec2 { x: 640.0, y: 320.0 }),
//         image_mode: SpriteImageMode::Sliced(TextureSlicer {
//             border: BorderRect::all(32.0),
//             center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
//             sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1.0 },
//             max_corner_scale: 1.0,
//         }),
//         ..default()
//     },));
// }

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_translation(vec3(0.0, 0.0, 100.0))));
}
