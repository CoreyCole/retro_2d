use crate::config::AppState;
use bevy::asset::LoadState;
use bevy::prelude::*;

pub struct AssetsPlugin;

#[derive(Clone)]
pub struct ImageAsset {
    pub handle: Handle<Image>,
    pub width: f32,
    pub height: f32,
}

impl From<&ImageAsset> for Handle<Image> {
    fn from(asset: &ImageAsset) -> Self {
        asset.handle.clone()
    }
}

impl From<ImageAsset> for Handle<Image> {
    fn from(asset: ImageAsset) -> Self {
        asset.handle
    }
}

impl ImageAsset {
    pub fn new(handle: Handle<Image>, assets: &Assets<Image>) -> Self {
        if let Some(image) = assets.get(&handle) {
            Self {
                handle,
                width: image.width() as f32,
                height: image.height() as f32,
            }
        } else {
            // Provide default dimensions if the image isn't loaded yet
            Self {
                handle,
                width: 64.0,
                height: 64.0,
            }
        }
    }
}

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
            &self.hoodie_selected,
            &self.transparent_rope,
        ]
        .into_iter()
    }

    pub fn get_dimensions(&self, handle: &Handle<Image>, assets: &Assets<Image>) -> (f32, f32) {
        if let Some(image) = assets.get(handle) {
            (image.width() as f32, image.height() as f32)
        } else {
            // Return default dimensions if asset isn't loaded yet
            (64.0, 64.0)
        }
    }
}

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::AssetsLoading);
        app.add_systems(Startup, load_startup_assets);
        app.add_systems(
            Update,
            check_assets_loaded.run_if(in_state(AppState::AssetsLoading)),
        );
    }
}

fn load_startup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load assets
    let retro2d_assets = Retro2dAssets {
        cows_and_basket: asset_server.load("cows_and_basket.png"),
        hoodie: asset_server.load("hoodie.png"),
        hoodie_glow: asset_server.load("hoodie_glow.png"),
        hoodie_selected: asset_server.load("hoodie_selected.png"),
        transparent_rope: asset_server.load("transparent_rope.png"),
    };

    commands.insert_resource(retro2d_assets);
}

fn check_assets_loaded(
    mut state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    retro2d_assets: Res<Retro2dAssets>,
) {
    // Check if all assets are loaded
    let mut all_loaded = true;
    let mut any_failed = false;

    for handle in retro2d_assets.iter() {
        match asset_server.get_load_state(handle) {
            Some(LoadState::Loaded) => continue,
            Some(LoadState::NotLoaded) | Some(LoadState::Loading) => {
                all_loaded = false;
                break;
            }
            Some(LoadState::Failed(_)) => {
                any_failed = true;
                all_loaded = false;
                break;
            }
            None => {
                all_loaded = false;
                break;
            }
        }
    }

    // Proceed if all assets are loaded or if we're on web and at least some assets loaded
    #[cfg(target_arch = "wasm32")]
    if all_loaded || any_failed {
        state.set(AppState::Game);
        return;
    }

    // On native, only proceed if all assets loaded
    #[cfg(not(target_arch = "wasm32"))]
    if all_loaded {
        state.set(AppState::Game);
    }
}
