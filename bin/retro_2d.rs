use bevy::prelude::*;
use bevy::window::PresentMode;
use retro_2d_lib::{AssetsPlugin, WorldPlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(AssetsPlugin)
        .add_plugins(WorldPlugin)
        .run();
}
