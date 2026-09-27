use tauri::{
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    Manager, Wry,
};

const PLUGIN_IDENTIFIER: &str = "com.ninelfx.hoshi";

pub struct PlayerSurfacePlugin(#[allow(dead_code)] pub PluginHandle<Wry>);

pub fn init() -> TauriPlugin<Wry> {
    Builder::new("player-surface")
        .setup(|app, api: PluginApi<Wry, ()>| {
            crate::player_surface::android::attach(app);

            let handle = api
                .register_android_plugin(PLUGIN_IDENTIFIER, "PlayerSurfacePlugin")
                .expect("Failed to register PlayerSurfacePlugin on Android");

            app.manage(PlayerSurfacePlugin(handle));
            Ok(())
        })
        .build()
}