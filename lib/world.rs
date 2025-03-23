use bevy::prelude::*;

use crate::assets::ImageAsset;
use crate::{
    AppState, DragPlugin, Draggable, DropStrategy, Group, Interactable, InteractionPlugin,
    InteractionSource, InteractionState, Retro2dAssets,
};

pub struct WorldPlugin;

const BG_GROUP: u8 = 0;
const ITEM_GROUP: u8 = 1;
const ROPE_SPACING: f32 = 400.0; // Distance between rope segments
const NUM_ROPES: i32 = 9; // Number of rope segments to create

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Rope {
    attached_to: Entity,
    offset: Vec2,
}

#[derive(Component, Clone)]
struct ItemState {
    normal: ImageAsset,
    glow: ImageAsset,
    selected: ImageAsset,
    is_glowing: bool,
    is_dragging: bool,
    is_selected: bool,
}

#[derive(Component, Clone)]
struct InitialTransform {
    translation: Vec3,
    scale: Vec3,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InteractionPlugin, DragPlugin));
        app.add_systems(OnExit(AppState::AssetsLoading), setup_background);
        app.add_systems(OnEnter(AppState::Game), setup_clothes);
        app.add_systems(
            Update,
            (
                interact_with_no_hover,
                interact_with_items,
                update_rope_position,
                setup_sprite_transforms,
            )
                .run_if(in_state(AppState::Game)),
        );
    }
}

fn interact_with_no_hover(
    interaction_state: Res<InteractionState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut items: Query<(Entity, &mut ItemState)>,
) {
    let mut no_hover = false;
    for (entity, _) in items.iter_mut() {
        let is_hovered = interaction_state
            .get_group(Group(ITEM_GROUP))
            .iter()
            .any(|(e, _)| *e == entity);
        no_hover |= is_hovered;
    }
    if !no_hover && mouse_button_input.just_pressed(MouseButton::Left) {
        println!("No hover");
        for (_, mut state) in items.iter_mut() {
            state.is_selected = false;
            state.is_glowing = false;
        }
    }
}

fn interact_with_items(
    interaction_state: Res<InteractionState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut items: Query<(Entity, &mut ItemState, &mut Sprite)>,
) {
    for (entity, mut state, mut sprite) in items.iter_mut() {
        let is_hovered = interaction_state
            .get_group(Group(ITEM_GROUP))
            .iter()
            .any(|(e, _)| *e == entity);

        // selection
        if !state.is_selected && mouse_button_input.just_pressed(MouseButton::Left) && is_hovered {
            state.is_selected = true;
            sprite.image = state.selected.handle.clone();
            println!("Selected");
        } else if mouse_button_input.just_pressed(MouseButton::Left) && !is_hovered {
            state.is_selected = false;
            sprite.image = state.normal.handle.clone();
            println!("Unselected")
        }
        // dragging
        if mouse_button_input.pressed(MouseButton::Left) && !state.is_dragging && is_hovered {
            state.is_dragging = true;
            println!("Dragging");
        } else if mouse_button_input.just_released(MouseButton::Left) {
            state.is_dragging = false;
            println!("Not dragging");
        }
        // hover glow
        if !state.is_glowing && is_hovered {
            sprite.image = state.glow.handle.clone();
            state.is_glowing = true;
            println!("Glowing");
        } else if state.is_glowing && !state.is_selected && !is_hovered {
            sprite.image = state.normal.handle.clone();
            state.is_glowing = false;
            println!("Not glowing");
        }
    }
}

fn setup_sprite_transforms(
    mut commands: Commands,
    mut sprites: Query<(Entity, &InitialTransform), Added<Sprite>>,
) {
    for (entity, initial_transform) in sprites.iter_mut() {
        commands.entity(entity).insert(Transform {
            translation: initial_transform.translation,
            scale: initial_transform.scale,
            ..Default::default()
        });
        commands.entity(entity).remove::<InitialTransform>();
    }
}

fn setup_clothes(
    mut commands: Commands,
    retro2d_assets: Res<Retro2dAssets>,
    assets: Res<Assets<Image>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();

    let hoodie = ImageAsset::new(retro2d_assets.hoodie.clone(), &assets);
    let rope = ImageAsset::new(retro2d_assets.transparent_rope.clone(), &assets);

    // Setup hoodie
    let hoodie_sprite = Sprite {
        image: hoodie.handle.clone(),
        ..Default::default()
    };

    let hoodie_initial_transform = InitialTransform {
        translation: Vec3::new(0.0, 0.0, 1.0),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };

    let hoodie_state = ItemState {
        normal: hoodie.clone(),
        glow: ImageAsset::new(retro2d_assets.hoodie_glow.clone(), &assets),
        selected: ImageAsset::new(retro2d_assets.hoodie_selected.clone(), &assets),
        is_glowing: false,
        is_dragging: false,
        is_selected: false,
    };

    let interactable = Interactable {
        groups: vec![Group(ITEM_GROUP)],
        bounding_box: (
            Vec2::new(-hoodie.width / 2.0, -hoodie.height / 2.0),
            Vec2::new(hoodie.width / 2.0, hoodie.height / 2.0),
        ),
    };

    let draggable = Draggable {
        groups: vec![Group(ITEM_GROUP)],
        hook: None,
        drop_strategy: DropStrategy::Leave,
        lock_y: true,
    };

    // Setup camera
    commands.spawn(Camera2d).insert(InteractionSource {
        groups: vec![
            Group(BG_GROUP),
            Group(ITEM_GROUP),
        ],
        ..Default::default()
    });

    // Spawn hoodie as parent entity
    let hoodie_entity = commands
        .spawn((
            hoodie_sprite.clone(),
            hoodie_initial_transform.clone(),
            hoodie_state.clone(),
            interactable.clone(),
            draggable.clone(),
        ))
        .id();

    // Calculate rope offset based on hoodie height
    let rope_offset = hoodie.height / 2. - 20.;

    // Create multiple ropes across the screen
    let start_x = -(window_width as f32 / 2.0) * ROPE_SPACING;

    commands
        .spawn((
            Transform {
                scale: Vec3::new(1., 1., 1.),
                ..Default::default()
            },
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                hoodie_sprite,
                hoodie_initial_transform,
                hoodie_state,
                interactable,
                draggable,
            ));
            for i in 0..NUM_ROPES {
                let x_pos = start_x + (i as f32 * ROPE_SPACING) - (window_width / 2.0);
                let rope_sprite = Sprite {
                    image: rope.handle.clone(),
                    ..Default::default()
                };
                let rope_initial_transform = InitialTransform {
                    translation: Vec3::new(x_pos, rope_offset, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                parent.spawn((
                    rope_sprite,
                    rope_initial_transform,
                    Rope {
                        attached_to: hoodie_entity,
                        offset: Vec2::new(x_pos, rope_offset),
                    },
                ));
            }
        });
}

fn update_rope_position(
    mut ropes: Query<(&mut Transform, &Rope)>,
    items: Query<&Transform, (With<ItemState>, Without<Rope>)>,
) {
    for (mut rope_transform, rope) in ropes.iter_mut() {
        if let Ok(item_transform) = items.get(rope.attached_to) {
            let item_x = item_transform.translation.x;

            // Calculate the base position relative to the hoodie
            let mut rope_x = item_x + rope.offset.x;

            // Wrap the rope when it goes too far from the center
            let total_width = ROPE_SPACING * (NUM_ROPES as f32);
            let wrap_threshold = total_width / 2.0;

            // Wrap to the left side when too far right
            while rope_x - item_x > wrap_threshold {
                rope_x -= total_width;
            }

            // Wrap to the right side when too far left
            while rope_x - item_x < -wrap_threshold {
                rope_x += total_width;
            }

            rope_transform.translation.x = rope_x;
            rope_transform.translation.y = item_transform.translation.y + rope.offset.y;
        }
    }
}

fn setup_background(
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

    let sprite = Sprite {
        image: background.handle.clone(),
        ..Default::default()
    };

    let initial_transform = InitialTransform {
        translation: Vec3::new(0.0, 0.0, -100.0),
        scale: Vec3::new(scale, scale, 1.0),
    };

    commands
        .spawn((sprite, initial_transform))
        .insert(Background);
}
