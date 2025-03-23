use crate::AppState;
use crate::{DragPlugin, InteractionPlugin};
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InteractionPlugin, DragPlugin));
        app.add_systems(
            OnExit(AppState::AssetsLoading),
            crate::world::bg::setup_background,
        );
        app.add_systems(
            OnEnter(AppState::Game),
            crate::world::clothes::setup_clothes,
        );
        app.add_systems(
            Update,
            (
                crate::world::clothes::interact_with_no_hover,
                crate::world::clothes::interact_with_items,
                crate::world::clothes::update_rope_position,
            )
                .run_if(in_state(AppState::Game)),
        );
    }
}
