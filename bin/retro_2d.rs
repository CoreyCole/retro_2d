use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use retro_2d_lib::{AssetsPlugin, WorldPlugin};

fn main() {
    let mut app = App::new();

    // Add the embedded assets plugin before DefaultPlugins
    app.add_plugins(EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
    });

    // Set up web-specific features
    #[cfg(target_arch = "wasm32")]
    {
        // Set panic hook for better error messages
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        // Initialize console logger
        console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
        // Use wee_alloc as the global allocator to reduce code size
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }

    // Configure platform-specific plugins
    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        fit_canvas_to_parent: true,
                        canvas: Some("#bevy-canvas".to_string()),
                        resolution: (800.0, 600.0).into(),
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(false),
                    mode: AssetMode::Unprocessed,
                    ..default()
                }),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync,
                        resolution: (1280.0, 900.0).into(),
                        title: "Retro 2D Game".to_string(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    mode: AssetMode::Unprocessed,
                    ..default()
                }),
        );
    }

    app.add_plugins(AssetsPlugin).add_plugins(WorldPlugin).run();
}
