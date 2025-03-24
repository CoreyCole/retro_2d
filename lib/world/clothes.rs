use crate::assets::{ImageAsset, Retro2dAssets};
use crate::{Draggable, DropStrategy, Group, Interactable, InteractionSource, InteractionState};
use bevy::prelude::*;

const ITEM_GROUP: u8 = 1;
const ROPE_SPACING: f32 = 400.0;
const NUM_ROPES: i32 = 9;

#[derive(Component)]
pub struct Rope {
    pub attached_to: Entity,
    pub offset: Vec2,
}

#[derive(Component, Clone)]
pub struct ItemState {
    pub normal: ImageAsset,
    pub glow: ImageAsset,
    pub selected: ImageAsset,
    pub is_glowing: bool,
    pub is_dragging: bool,
    pub is_selected: bool,
}

pub fn interact_with_no_hover(
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
        for (_, mut state) in items.iter_mut() {
            state.is_selected = false;
            state.is_glowing = false;
        }
    }
}

pub fn interact_with_items(
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
        } else if mouse_button_input.just_pressed(MouseButton::Left) && !is_hovered {
            state.is_selected = false;
            sprite.image = state.normal.handle.clone();
        }
        // dragging
        if mouse_button_input.pressed(MouseButton::Left) && !state.is_dragging && is_hovered {
            state.is_dragging = true;
        } else if mouse_button_input.just_released(MouseButton::Left) {
            state.is_dragging = false;
        }
        // hover glow
        if !state.is_glowing && is_hovered {
            sprite.image = state.glow.handle.clone();
            state.is_glowing = true;
        } else if state.is_glowing && !state.is_selected && !is_hovered {
            sprite.image = state.normal.handle.clone();
            state.is_glowing = false;
        }
    }
}

pub fn setup_clothes(
    mut commands: Commands,
    retro2d_assets: Res<Retro2dAssets>,
    assets: Res<Assets<Image>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();

    let hoodie = ImageAsset::new(retro2d_assets.hoodie.clone(), &assets);
    let rope = ImageAsset::new(retro2d_assets.transparent_rope.clone(), &assets);

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
            Group(0), // BG_GROUP
            Group(ITEM_GROUP),
        ],
        ..Default::default()
    });

    // Spawn hoodie as parent entity
    let hoodie_entity = commands
        .spawn((
            Sprite {
                image: hoodie.handle.clone(),
                ..Default::default()
            },
            Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                ..Default::default()
            },
            hoodie_state.clone(),
            interactable.clone(),
            draggable.clone(),
        ))
        .id();

    // Calculate rope offset based on hoodie height
    let rope_offset = hoodie.height / 2.0 - 20.;

    // Create multiple ropes across the screen
    let start_x = -(window_width as f32 / 2.0) * ROPE_SPACING;

    // Spawn ropes
    commands
        .spawn((
            Transform {
                scale: Vec3::new(1., 1., 1.),
                ..Default::default()
            },
            Visibility::default(),
        ))
        .with_children(|parent| {
            for i in 0..NUM_ROPES {
                let x_pos = start_x + (i as f32 * ROPE_SPACING);
                let rope_sprite = Sprite {
                    image: rope.handle.clone(),
                    ..Default::default()
                };
                parent.spawn((
                    rope_sprite,
                    Transform {
                        translation: Vec3::new(x_pos, rope_offset, 1.0),
                        ..Default::default()
                    },
                    Rope {
                        attached_to: hoodie_entity,
                        offset: Vec2::new(x_pos, rope_offset),
                    },
                ));
            }
        });
}

pub fn update_rope_position(
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
