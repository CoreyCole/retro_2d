use bevy::prelude::*;

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

#[derive(Component)]
struct ItemState {
    normal: Handle<Image>,
    glow: Handle<Image>,
    selected: Handle<Image>,
    is_glowing: bool,
    is_dragging: bool,
    is_selected: bool,
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
            )
                .run_if(in_state(AppState::Game)),
        );
    }
}

fn interact_with_no_hover(
    interaction_state: Res<InteractionState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut items: Query<(Entity, &mut Handle<Image>, &mut ItemState)>,
) {
    let mut no_hover = false;
    for (entity, _, _) in items.iter_mut() {
        let is_hovered = interaction_state
            .get_group(Group(ITEM_GROUP))
            .iter()
            .any(|(e, _)| *e == entity);
        no_hover |= is_hovered;
    }
    if !no_hover && mouse_button_input.just_pressed(MouseButton::Left) {
        println!("No hover");
        for (_, mut texture, mut state) in items.iter_mut() {
            state.is_selected = false;
            state.is_glowing = false;
            *texture = state.normal.clone();
        }
    }
}

fn interact_with_items(
    interaction_state: Res<InteractionState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut items: Query<(Entity, &mut Handle<Image>, &mut ItemState)>,
) {
    for (entity, mut texture, mut state) in items.iter_mut() {
        let is_hovered = interaction_state
            .get_group(Group(ITEM_GROUP))
            .iter()
            .any(|(e, _)| *e == entity);

        // selection
        if !state.is_selected && mouse_button_input.just_pressed(MouseButton::Left) && is_hovered {
            state.is_selected = true;
            *texture = state.selected.clone();
            println!("Selected");
        } else if mouse_button_input.just_pressed(MouseButton::Left) && !is_hovered {
            state.is_selected = false;
            *texture = state.normal.clone();
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
            *texture = state.glow.clone();
            state.is_glowing = true;
            println!("Glowing");
        } else if state.is_glowing && !state.is_selected && !is_hovered {
            *texture = state.normal.clone();
            state.is_glowing = false;
            println!("Not glowing");
        }
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

    let hoodie = retro2d_assets.hoodie.clone();
    let rope = retro2d_assets.transparent_rope.clone();

    // Setup hoodie
    let hoodie_bundle = SpriteBundle {
        texture: hoodie.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let hoodie_state = ItemState {
        normal: hoodie.clone(),
        glow: retro2d_assets.hoodie_glow.clone(),
        selected: retro2d_assets.hoodie_selected.clone(),
        is_glowing: false,
        is_dragging: false,
        is_selected: false,
    };

    let (width, height) = retro2d_assets.get_dimensions(&hoodie, &assets);
    let bounding_box = Vec2::new(width, height);
    let interactable = Interactable {
        groups: vec![Group(ITEM_GROUP)],
        bounding_box: (
            Vec2::new(-bounding_box.x / 2.0, -bounding_box.y / 2.0),
            Vec2::new(bounding_box.x / 2.0, bounding_box.y / 2.0),
        ),
    };
    let draggable = Draggable {
        groups: vec![Group(ITEM_GROUP)],
        hook: None,
        drop_strategy: DropStrategy::Leave,
        lock_y: true,
    };

    // Setup camera
    commands
        .spawn(Camera2dBundle::default())
        .insert(InteractionSource {
            groups: vec![
                Group(BG_GROUP),
                Group(ITEM_GROUP),
            ],
            ..Default::default()
        });

    // Spawn hoodie
    let item = commands
        .spawn((hoodie_bundle, hoodie_state))
        .insert(interactable)
        .insert(draggable)
        .id();

    // Calculate the offset from the hoodie's center to its top
    let hoodie_half_height = height / 2.0;
    let rope_offset = hoodie_half_height - 20.0;

    // Create multiple ropes across the screen
    let start_x = -(window_width as f32 / 2.0) * ROPE_SPACING;
    let mut rope_entities = Vec::new();

    for i in 0..NUM_ROPES {
        let x_pos = start_x + (i as f32 * ROPE_SPACING);

        let rope_bundle = SpriteBundle {
            texture: rope.clone(),
            transform: Transform {
                translation: Vec3::new(x_pos, rope_offset, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        };

        let rope_entity = commands
            .spawn(rope_bundle)
            .insert(Rope {
                attached_to: item,
                offset: Vec2::new(x_pos, rope_offset),
            })
            .id();

        rope_entities.push(rope_entity);
    }

    // Create parent entity for organization
    commands
        .spawn(SpatialBundle::from_transform(Transform {
            scale: Vec3::new(1., 1., 1.),
            ..Default::default()
        }))
        .push_children(&[item])
        .push_children(&rope_entities);
}

fn update_rope_position(
    mut ropes: Query<(&mut Transform, &mut Rope)>,
    items: Query<&Transform, Without<Rope>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();
    let half_width = window_width / 2.0;

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
