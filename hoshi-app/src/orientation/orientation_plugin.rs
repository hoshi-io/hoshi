use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginApi, PluginHandle, TauriPlugin},
    plugin::mobile::PluginInvokeError,
    Manager, Runtime,
};
use tracing::debug;

const PLUGIN_IDENTIFIER: &str = "com.ninelfx.hoshi";


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Portrait,
    PortraitReverse,
    Landscape,
    LandscapeReverse,
    SensorPortrait,
    SensorLandscape,
    Sensor,
    Unspecified,
}

impl Orientation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Orientation::Portrait         => "portrait",
            Orientation::PortraitReverse  => "portrait_reverse",
            Orientation::Landscape        => "landscape",
            Orientation::LandscapeReverse => "landscape_reverse",
            Orientation::SensorPortrait   => "sensor_portrait",
            Orientation::SensorLandscape  => "sensor_landscape",
            Orientation::Sensor           => "sensor",
            Orientation::Unspecified      => "unspecified",
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LockPayload {
    orientation: String,
}

#[derive(Serialize, Deserialize)]
pub struct Empty {}

#[derive(Deserialize)]
pub struct CurrentOrientationResponse {
    pub orientation: String,
}


pub struct OrientationPlugin<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> OrientationPlugin<R> {
    pub fn lock(&self, orientation: Orientation) -> Result<(), PluginInvokeError> {
        debug!(orientation = %orientation.as_str(), "Dispatching lock to Android OrientationPlugin");
        self.0
            .run_mobile_plugin::<Empty>("lock", LockPayload {
                orientation: orientation.as_str().to_string(),
            })
            .map(|_| ())
    }

    pub fn unlock(&self) -> Result<(), PluginInvokeError> {
        debug!("Dispatching unlock to Android OrientationPlugin");
        self.0
            .run_mobile_plugin::<Empty>("unlock", Empty {})
            .map(|_| ())
    }

    pub fn get_current(&self) -> Result<String, PluginInvokeError> {
        debug!("Dispatching getCurrent to Android OrientationPlugin");
        self.0
            .run_mobile_plugin::<CurrentOrientationResponse>("getCurrent", Empty {})
            .map(|r| r.orientation)
    }
}

pub trait OrientationPluginExt<R: Runtime> {
    fn orientation_plugin(&self) -> tauri::State<'_, OrientationPlugin<R>>;
}

impl<R: Runtime, T: Manager<R>> OrientationPluginExt<R> for T {
    fn orientation_plugin(&self) -> tauri::State<'_, OrientationPlugin<R>> {
        self.state::<OrientationPlugin<R>>()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("orientation")
        .invoke_handler(tauri::generate_handler![
            lock_orientation,
            unlock_orientation,
            get_current_orientation,
        ])
        .setup(|app, api: PluginApi<R, ()>| {
            let handle = api
                .register_android_plugin(PLUGIN_IDENTIFIER, "OrientationPlugin")
                .expect("Failed to register OrientationPlugin on Android");

            app.manage(OrientationPlugin(handle));
            Ok(())
        })
        .build()
}

#[tauri::command]
pub fn lock_orientation<R: Runtime>(
    app: tauri::AppHandle<R>,
    orientation: String,
) -> Result<(), String> {
    println!("LLAMADA ORIENTACION: {}", orientation);
    let o = parse_orientation(&orientation)
        .ok_or_else(|| format!("Orientación desconocida: {}", orientation))?;

    app.orientation_plugin()
        .lock(o)
        .map_err(|e| format!("Error al bloquear orientación: {}", e))
}

#[tauri::command]
pub fn unlock_orientation<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    app.orientation_plugin()
        .unlock()
        .map_err(|e| format!("Error al desbloquear orientación: {}", e))
}

#[tauri::command]
pub fn get_current_orientation<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<String, String> {
    app.orientation_plugin()
        .get_current()
        .map_err(|e| format!("Error al obtener orientación: {}", e))
}

fn parse_orientation(s: &str) -> Option<Orientation> {
    match s {
        "portrait"          => Some(Orientation::Portrait),
        "portrait_reverse"  => Some(Orientation::PortraitReverse),
        "landscape"         => Some(Orientation::Landscape),
        "landscape_reverse" => Some(Orientation::LandscapeReverse),
        "sensor_portrait"   => Some(Orientation::SensorPortrait),
        "sensor_landscape"  => Some(Orientation::SensorLandscape),
        "sensor"            => Some(Orientation::Sensor),
        "unspecified"       => Some(Orientation::Unspecified),
        _                   => None,
    }
}