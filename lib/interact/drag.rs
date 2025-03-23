use bevy::prelude::*;

use crate::{Group, Interactable, InteractionState};

#[derive(Component)]
pub struct Dragged {
    pub group: Group,
    pub translation: Vec2,
    pub origin: Vec2,
    pub just_dropped: bool,
    pub just_dragged: bool,
}

impl Dragged {
    pub fn just_dropped(&self) -> bool {
        self.just_dropped
    }
    pub fn just_dragged(&self) -> bool {
        self.just_dragged
    }
}

pub struct DragPlugin;
impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionState>().add_systems(
            Update,
            (
                mouse_press_start_drag_system,
                mouse_release_stop_drag_system,
                drag_system,
            ),
        );
    }
}

pub fn drag_system(
    interaction_state: Res<InteractionState>,
    mut draggables: Query<(&mut Transform, &mut Dragged, &Draggable, &GlobalTransform)>,
) {
    for (mut transform, mut dragged, draggable, global_transform) in draggables.iter_mut() {
        if dragged.just_dragged {
            dragged.just_dragged = false;
        }
        if let Some(cursor_position) = interaction_state.cursor_positions.get(&dragged.group) {
            let new_y = if draggable.lock_y {
                dragged.origin.y
            } else {
                cursor_position.y + dragged.translation.y
            };
            let parent_matrix = global_transform
                .compute_matrix()
                .mul_mat4(&transform.compute_matrix().inverse());
            let global_hook_translation =
                Vec2::new(cursor_position.x + dragged.translation.x, new_y)
                    .extend(transform.translation.z);

            transform.translation = parent_matrix
                .inverse()
                .transform_point3(global_hook_translation);
        }
    }
}

#[derive(Clone)]
pub enum DropStrategy {
    Reset,
    Leave,
}

#[derive(Component, Clone)]
pub struct Draggable {
    // Where the entity is hooked onto the cursor while dragging.
    // If no hook is given, the entity will be pinned to the cursor
    // as it was when the drag was started.
    pub hook: Option<Vec2>,
    pub groups: Vec<Group>,
    pub drop_strategy: DropStrategy,
    pub lock_y: bool,
}

impl Default for Draggable {
    fn default() -> Self {
        Self {
            hook: None,
            groups: vec![Group::default()],
            drop_strategy: DropStrategy::Leave,
            lock_y: false,
        }
    }
}

pub fn mouse_press_start_drag_system(
    interaction_state: Res<InteractionState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    draggables: Query<(Entity, &Draggable, &GlobalTransform), With<Interactable>>,
    mut commands: Commands,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }
    for (entity, draggable, global_transform) in draggables.iter() {
        for group in draggable.groups.iter() {
            if let Some(list) = interaction_state.ordered_interact_list_map.get(group) {
                if let Some((_, position)) = list.iter().find(|(e, _)| e == &entity) {
                    let translation = draggable
                        .hook
                        .unwrap_or(global_transform.translation().truncate() - *position);
                    commands.entity(entity).insert(Dragged {
                        group: *group,
                        translation,
                        origin: global_transform.translation().truncate(),
                        just_dropped: false,
                        just_dragged: true,
                    });
                    break;
                }
            }
        }
    }
}

pub fn mouse_release_stop_drag_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut draggables: Query<(Entity, &Draggable, &mut Dragged, &mut Transform), With<Interactable>>,
    mut commands: Commands,
) {
    if !mouse_button_input.just_released(MouseButton::Left) {
        return;
    }
    for (entity, draggable, mut dragged, mut transform) in draggables.iter_mut() {
        if !dragged.just_dragged {
            dragged.just_dropped = true;
        }
        if let DropStrategy::Reset = draggable.drop_strategy {
            transform.translation = dragged.origin.extend(transform.translation.z);
        }
        commands.entity(entity).remove::<Dragged>();
    }
}
