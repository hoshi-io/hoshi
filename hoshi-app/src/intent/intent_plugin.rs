use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    plugin::mobile::PluginInvokeError,
    Manager, Runtime,
};

const PLUGIN_IDENTIFIER: &str = "com.dot_fx.hoshi";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchIntentPayload {
    url: String,
    title: String,
    position: i32,
    subs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Empty {}

pub struct IntentPlugin<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> IntentPlugin<R> {
    pub fn launch_intent(
        &self,
        url: String,
        title: String,
        position: i32,
        subs: Vec<String>,
    ) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin::<Empty>("launchIntent", LaunchIntentPayload { url, title, position, subs })
            .map(|_| ())
    }
}

pub trait IntentPluginExt<R: Runtime> {
    fn intent_plugin(&self) -> tauri::State<'_, IntentPlugin<R>>;
}

impl<R: Runtime, T: Manager<R>> IntentPluginExt<R> for T {
    fn intent_plugin(&self) -> tauri::State<'_, IntentPlugin<R>> {
        self.state::<IntentPlugin<R>>()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("intent")
        .invoke_handler(tauri::generate_handler![launch_intent])
        .setup(|app, api: PluginApi<R, ()>| {
            let handle = api
                .register_android_plugin(PLUGIN_IDENTIFIER, "IntentPlugin")
                .expect("Failed to register IntentPlugin on Android");

            app.manage(IntentPlugin(handle));
            Ok(())
        })
        .build()
}

#[tauri::command]
pub fn launch_intent<R: Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
    title: Option<String>,
    position: Option<i32>,
    subs: Option<Vec<String>>,
) -> Result<(), String> {
    app.intent_plugin()
        .launch_intent(
            url,
            title.unwrap_or_default(),
            position.unwrap_or(0),
            subs.unwrap_or_default(),
        )
        .map_err(|e| format!("Failed to launch intent: {}", e))
}