use crate::assets::{ImageAsset, Retro2dAssets};
use bevy::prelude::*;

#[derive(Component)]
pub struct Background;

pub fn setup_background(
    mut commands: Commands,
    retro2d_assets: Res<Retro2dAssets>,
    windows: Query<&Window>,
    assets: Res<Assets<Image>>,
) {
    let window = windows.single();
    let background = ImageAsset::new(retro2d_assets.cows_and_basket.clone(), &assets);
    let window_aspect_ratio = window.width() / window.height();
    let image_width = background.width;
    let image_height = background.height;

    let image_aspect_ratio = image_width / image_height;
    let scale = if window_aspect_ratio > image_aspect_ratio {
        window.height() / image_height
    } else {
        window.width() / image_width
    };

    commands
        .spawn((
            Sprite {
                image: background.handle.clone(),
                ..Default::default()
            },
            Transform {
                scale: Vec3::new(scale, scale, 1.0),
                ..Default::default()
            },
        ))
        .insert(Background);
}
