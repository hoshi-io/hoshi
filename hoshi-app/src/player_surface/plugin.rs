use tauri::{
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    Manager, Wry,
};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.ninelfx.hoshi";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_player_surface);

pub struct PlayerSurfacePlugin(#[allow(dead_code)] pub PluginHandle<Wry>);

pub fn init() -> TauriPlugin<Wry> {
    Builder::new("player-surface")
        .setup(|app, api: PluginApi<Wry, ()>| {
            crate::player_surface::attach(app);

            #[cfg(target_os = "android")]
            let handle = api
                .register_android_plugin(PLUGIN_IDENTIFIER, "PlayerSurfacePlugin")
                .expect("Failed to register PlayerSurfacePlugin on Android");
            #[cfg(target_os = "ios")]
            let handle = api
                .register_ios_plugin(init_plugin_player_surface)
                .expect("Failed to register PlayerSurfacePlugin on iOS");

            app.manage(PlayerSurfacePlugin(handle));
            Ok(())
        })
        .build()
}
