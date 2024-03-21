use bevy::prelude::*;

use crate::{assets::Retro2dAssets, config::AppState};

pub struct WorldPlugin;

#[derive(Component)]
struct Background;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_background);
    }
}

fn setup_background(
    mut commands: Commands,
    retro2d_assets: Res<Retro2dAssets>,
    windows: Query<&Window>,
    assets: Res<Assets<Image>>,
) {
    let window = windows.single();
    let image = assets.get(retro2d_assets.cows_and_basket.clone()).unwrap();
    let window_aspect_ratio = window.width() / window.height();
    let image_width = image.width() as f32;
    let image_height = image.height() as f32;

    let image_aspect_ratio = image_width / image_height;
    let scale = if window_aspect_ratio > image_aspect_ratio {
        window.height() / image_height
    } else {
        window.width() / image_width
    };

    let sprite_bundle = SpriteBundle {
        texture: retro2d_assets.cows_and_basket.clone(),
        transform: Transform {
            scale: Vec3::new(scale, scale, 1.),
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    };

    commands.spawn(sprite_bundle).insert(Background);
}
