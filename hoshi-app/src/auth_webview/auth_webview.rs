use serde::{Deserialize, Serialize};
use tauri::{
    plugin::mobile::PluginInvokeError,
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    Manager, Runtime,
};

const PLUGIN_IDENTIFIER: &str = "com.ninelfx.hoshi";

#[derive(Serialize, Deserialize)]
pub struct Empty {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenArgs {
    url: String,
    redirect_prefixes: Vec<String>,
}

#[derive(Deserialize)]
struct OpenResult {
    /// Full redirect URL (fragment included). Missing when the user closed the window.
    url: Option<String>,
}

pub struct AuthWebViewPlugin<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> AuthWebViewPlugin<R> {
    /// Stays pending until the login redirects to one of `redirect_prefixes` (Some(url))
    /// or the user closes the window (None).
    pub async fn open(
        &self,
        url: String,
        redirect_prefixes: Vec<String>,
    ) -> Result<Option<String>, PluginInvokeError> {
        self.0
            .run_mobile_plugin_async::<OpenResult>(
                "open",
                OpenArgs {
                    url,
                    redirect_prefixes,
                },
            )
            .await
            .map(|r| r.url)
    }

    pub fn close(&self) -> Result<(), PluginInvokeError> {
        self.0
            .run_mobile_plugin::<Empty>("close", Empty {})
            .map(|_| ())
    }
}

pub trait AuthWebViewPluginExt<R: Runtime> {
    fn auth_webview_plugin(&self) -> tauri::State<'_, AuthWebViewPlugin<R>>;
}

impl<R: Runtime, T: Manager<R>> AuthWebViewPluginExt<R> for T {
    fn auth_webview_plugin(&self) -> tauri::State<'_, AuthWebViewPlugin<R>> {
        self.state::<AuthWebViewPlugin<R>>()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("authwebview")
        .invoke_handler(tauri::generate_handler![auth_webview_open, auth_webview_close])
        .setup(|app, api: PluginApi<R, ()>| {
            let handle = api
                .register_android_plugin(PLUGIN_IDENTIFIER, "AuthWebViewPlugin")
                .expect("Failed to register AuthWebViewPlugin on Android");

            app.manage(AuthWebViewPlugin(handle));
            Ok(())
        })
        .build()
}

// These MUST be async: the call waits for the user to finish logging in, and a sync command
// would block the thread it runs on for that whole time.
#[tauri::command]
pub async fn auth_webview_open<R: Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
    redirect_prefixes: Vec<String>,
) -> Result<Option<String>, String> {
    app.auth_webview_plugin()
        .open(url, redirect_prefixes)
        .await
        .map_err(|e| format!("Error opening login: {}", e))
}

#[tauri::command]
pub async fn auth_webview_close<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    app.auth_webview_plugin()
        .close()
        .map_err(|e| format!("Error closing login: {}", e))
}