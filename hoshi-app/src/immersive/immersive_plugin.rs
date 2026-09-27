use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    plugin::mobile::PluginInvokeError,
    Manager, Runtime,
};

const PLUGIN_IDENTIFIER: &str = "com.ninelfx.hoshi";

#[derive(Serialize, Deserialize)]
pub struct Empty {}

#[derive(Serialize)]
struct LevelArgs {
    level: f32,
}

#[derive(Deserialize)]
struct LevelResult {
    level: f32,
}

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

    pub fn set_brightness(&self, level: f32) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin::<Empty>("setBrightness", LevelArgs { level })
            .map(|_| ())
    }

    pub fn get_brightness(&self) -> Result<f32, PluginInvokeError> {
        self.0
            .run_mobile_plugin::<LevelResult>("getBrightness", Empty {})
            .map(|r| r.level)
    }

    pub fn set_volume(&self, level: f32) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin::<Empty>("setVolume", LevelArgs { level })
            .map(|_| ())
    }

    pub fn get_volume(&self) -> Result<f32, PluginInvokeError> {
        self.0
            .run_mobile_plugin::<LevelResult>("getVolume", Empty {})
            .map(|r| r.level)
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
            set_immersive_brightness,
            get_immersive_brightness,
            set_immersive_volume,
            get_immersive_volume,
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

#[tauri::command]
pub fn set_immersive_brightness<R: Runtime>(app: tauri::AppHandle<R>, level: f32) -> Result<(), String> {
    app.immersive_plugin()
        .set_brightness(level)
        .map_err(|e| format!("Error setting brightness: {}", e))
}

#[tauri::command]
pub fn get_immersive_brightness<R: Runtime>(app: tauri::AppHandle<R>) -> Result<f32, String> {
    app.immersive_plugin()
        .get_brightness()
        .map_err(|e| format!("Error getting brightness: {}", e))
}

#[tauri::command]
pub fn set_immersive_volume<R: Runtime>(app: tauri::AppHandle<R>, level: f32) -> Result<(), String> {
    app.immersive_plugin()
        .set_volume(level)
        .map_err(|e| format!("Error setting volume: {}", e))
}

#[tauri::command]
pub fn get_immersive_volume<R: Runtime>(app: tauri::AppHandle<R>) -> Result<f32, String> {
    app.immersive_plugin()
        .get_volume()
        .map_err(|e| format!("Error getting volume: {}", e))
}