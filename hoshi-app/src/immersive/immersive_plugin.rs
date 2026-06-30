use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    plugin::mobile::PluginInvokeError,
    Manager, Runtime,
};

const PLUGIN_IDENTIFIER: &str = "com.ninelfx.hoshi";

#[derive(Serialize, Deserialize)]
pub struct Empty {}

pub struct ImmersivePlugin<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> ImmersivePlugin<R> {
    pub fn enter(&self) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin::<Empty>("enter", Empty {})
            .map(|_| ())
    }

    pub fn exit(&self) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin::<Empty>("exit", Empty {})
            .map(|_| ())
    }
}

pub trait ImmersivePluginExt<R: Runtime> {
    fn immersive_plugin(&self) -> tauri::State<'_, ImmersivePlugin<R>>;
}

impl<R: Runtime, T: Manager<R>> ImmersivePluginExt<R> for T {
    fn immersive_plugin(&self) -> tauri::State<'_, ImmersivePlugin<R>> {
        self.state::<ImmersivePlugin<R>>()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("immersive")
        .invoke_handler(tauri::generate_handler![
            enter_fullscreen,
            exit_fullscreen,
        ])
        .setup(|app, api: PluginApi<R, ()>| {
            let handle = api
                .register_android_plugin(PLUGIN_IDENTIFIER, "ImmersivePlugin")
                .expect("Failed to register ImmersivePlugin on Android");

            app.manage(ImmersivePlugin(handle));
            Ok(())
        })
        .build()
}

#[tauri::command]
pub fn enter_fullscreen<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    app.immersive_plugin()
        .enter()
        .map_err(|e| format!("Error entering immersive mode: {}", e))
}

#[tauri::command]
pub fn exit_fullscreen<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    app.immersive_plugin()
        .exit()
        .map_err(|e| format!("Error exiting immersive mode: {}", e))
}