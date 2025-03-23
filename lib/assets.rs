use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::config::AppState;

pub struct AssetsPlugin;

#[derive(Resource)]
pub struct Retro2dAssets {
    pub cows_and_basket: Handle<Image>,
    pub hoodie: Handle<Image>,
    pub hoodie_glow: Handle<Image>,
    pub hoodie_selected: Handle<Image>,
    pub transparent_rope: Handle<Image>,
}

impl Retro2dAssets {
    pub fn iter(&self) -> impl Iterator<Item = &Handle<Image>> {
        vec![
            &self.cows_and_basket,
            &self.hoodie_glow,
            &self.hoodie,
            &self.transparent_rope,
        ]
        .into_iter()
    }
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::AssetsLoading);
        app.add_systems(Startup, load_startup_assets);
        app.add_systems(
            Update,
            { check_assets_loaded }.run_if(in_state(AppState::AssetsLoading)),
        );
    }
}

fn load_startup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = Retro2dAssets {
        cows_and_basket: asset_server.load("cows_and_basket.png"),
        hoodie: asset_server.load("hoodie.png"),
        hoodie_glow: asset_server.load("hoodie_glow.png"),
        hoodie_selected: asset_server.load("hoodie_selected.png"),
        transparent_rope: asset_server.load("transparent_rope.png"),
    };
    commands.insert_resource(assets);
}

fn check_assets_loaded(
    mut state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    retro2d_assets: Res<Retro2dAssets>,
) {
    for handle in retro2d_assets.iter() {
        let load_state = asset_server
            .get_load_state(handle)
            .unwrap_or(LoadState::NotLoaded);
        if load_state != LoadState::Loaded {
            return;
        }
    }
    println!("Assets loaded");
    state.set(AppState::Game);
}
