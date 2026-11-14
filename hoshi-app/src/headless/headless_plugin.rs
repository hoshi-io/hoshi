use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    plugin::mobile::PluginInvokeError,
    Manager, Runtime,
};
use tracing::{debug};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.ninelfx.hoshi";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_headless);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePayload {
    pub label:       String,
    pub url:         String,
    pub init_script: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DestroyPayload {
    pub label: String,
}

#[derive(Deserialize)]
pub struct Empty {}

pub struct HeadlessPlugin<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> HeadlessPlugin<R> {
    pub fn create(&self, payload: CreatePayload) -> Result<(), PluginInvokeError> {
        debug!(label = %payload.label, url = %payload.url, "Dispatching create to mobile HeadlessPlugin");
        self.0.run_mobile_plugin::<Empty>("create", payload).map(|_| ())
    }

    pub fn destroy(&self, label: &str) -> Result<(), PluginInvokeError> {
        debug!(label = %label, "Dispatching destroy to mobile HeadlessPlugin");
        self.0.run_mobile_plugin::<Empty>("destroy", DestroyPayload {
            label: label.to_string(),
        }).map(|_| ())
    }
}

pub trait HeadlessPluginExt<R: Runtime> {
    fn headless_plugin(&self) -> tauri::State<'_, HeadlessPlugin<R>>;
}

impl<R: Runtime, T: Manager<R>> HeadlessPluginExt<R> for T {
    fn headless_plugin(&self) -> tauri::State<'_, HeadlessPlugin<R>> {
        self.state::<HeadlessPlugin<R>>()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("headless")
        .invoke_handler(tauri::generate_handler![])
        .setup(|app, api: PluginApi<R, ()>| {
            #[cfg(target_os = "android")]
            let handle = api
                .register_android_plugin(PLUGIN_IDENTIFIER, "HeadlessPlugin")
                .expect("Failed to register HeadlessPlugin on Android");
            #[cfg(target_os = "ios")]
            let handle = api
                .register_ios_plugin(init_plugin_headless)
                .expect("Failed to register HeadlessPlugin on iOS");

            app.manage(HeadlessPlugin(handle));
            Ok(())
        })
        .build()
}

#[tauri::command]
pub fn notify_done(label: String, data: String) -> () {
    debug!(label = %label, "Received notify_done from mobile HeadlessPlugin");
    crate::headless::headless_sync::resolve_slot(&label, data);
}