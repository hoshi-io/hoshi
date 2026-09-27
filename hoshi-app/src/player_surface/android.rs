use std::sync::{Mutex, OnceLock};

use jni::objects::{GlobalRef, JClass, JObject};
use jni::JNIEnv;
use tauri::{AppHandle, Manager, Wry};
use tracing::{error, info, warn};

use hoshi_core::playback::RenderTarget;

static APP_HANDLE: OnceLock<AppHandle<Wry>> = OnceLock::new();

/// The `Surface` currently handed to mpv
static CURRENT_SURFACE: Mutex<Option<GlobalRef>> = Mutex::new(None);

static PENDING_SURFACE: Mutex<Option<(GlobalRef, i64)>> = Mutex::new(None);

use std::sync::Once;
use std::os::raw::{c_int, c_void};

unsafe extern "C" {
    fn av_jni_set_java_vm(vm: *mut c_void, log_ctx: *mut c_void) -> c_int;
}

static JVM_REGISTERED: Once = Once::new();

fn ensure_jvm_registered(env: &JNIEnv) {
    JVM_REGISTERED.call_once(|| {
        if let Ok(vm) = env.get_java_vm() {
            let vm_ptr = vm.get_java_vm_pointer() as *mut c_void;
            let ret = unsafe { av_jni_set_java_vm(vm_ptr, std::ptr::null_mut()) };
            if ret < 0 {
                error!("av_jni_set_java_vm failed: {ret}");
            } else {
                info!("registered JavaVM with ffmpeg/mpv android jni bridge");
            }
        } else {
            error!("failed to get JavaVM from JNIEnv");
        }
    });
}

pub fn attach(app: &AppHandle<Wry>) {
    info!("player_surface::attach() called");
    if APP_HANDLE.set(app.clone()).is_err() {
        warn!("android player_surface::attach called more than once, ignoring");
        return;
    }
    app.manage(crate::player_surface::PlayerReadyNotify(std::sync::Arc::new(
        tokio::sync::Notify::new(),
    )));
}

pub fn flush_pending_surface() {
    let Some((global, wid)) = PENDING_SURFACE.lock().unwrap().take() else {
        return;
    };
    let Some(state) = app_state() else {
        // Shouldn't happen — called right after app.manage(state) — but
        // don't lose the surface silently if it somehow does.
        warn!("flush_pending_surface: AppState still unavailable, re-queuing");
        *PENDING_SURFACE.lock().unwrap() = Some((global, wid));
        return;
    };

    info!(wid, "flushing pending Surface queued before AppState was ready");
    tauri::async_runtime::spawn(async move {
        let init_result = state
            .playback
            .initialize(RenderTarget::NativeWindow { wid, gpu_context: "android" })
            .await;

        if let Err(e) = &init_result {
            error!(?e, "failed to initialize mpv from pending surface");
        }

        // Notify waiters (e.g. wait_for_player_ready) regardless of
        // success, so a failure surfaces to the frontend instead of
        // hanging it forever.
        if let Some(app) = APP_HANDLE.get() {
            app.state::<crate::player_surface::PlayerReadyNotify>().0.notify_one();
        }

        if init_result.is_ok() {
            *CURRENT_SURFACE.lock().unwrap() = Some(global);
        }
    });
}

fn app_state() -> Option<std::sync::Arc<hoshi_core::AppState>> {
    APP_HANDLE
        .get()
        .and_then(|app| app.try_state::<std::sync::Arc<hoshi_core::AppState>>())
        .map(|s| s.inner().clone())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_ninelfx_hoshi_PlayerSurfaceView_nativeSurfaceCreated<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    surface: JObject<'local>,
) {
    ensure_jvm_registered(&env);
    let global = match env.new_global_ref(&surface) {
        Ok(g) => g,
        Err(e) => {
            error!(?e, "failed to take global ref on incoming Surface");
            return;
        }
    };
    let wid = global.as_raw() as usize as i64;

    let Some(state) = app_state() else {
        warn!(wid, "nativeSurfaceCreated fired before AppState was ready, queuing");
        *PENDING_SURFACE.lock().unwrap() = Some((global, wid));
        return;
    };

    tauri::async_runtime::spawn(async move {
        let result = if state.playback.mpv_handle().await.is_ok() {
            state.playback.attach_surface(wid).await
        } else {
            let init_result = state
                .playback
                .initialize(RenderTarget::NativeWindow { wid, gpu_context: "android" })
                .await;
            if init_result.is_ok() {
                if let Some(app) = APP_HANDLE.get() {
                    app.state::<crate::player_surface::PlayerReadyNotify>().0.notify_one();
                }
            }
            init_result
        };

        if let Err(e) = result {
            error!(?e, "failed to attach android surface to mpv");
            if let Some(app) = APP_HANDLE.get() {
                app.state::<crate::player_surface::PlayerReadyNotify>().0.notify_one();
            }
            return;
        }

        *CURRENT_SURFACE.lock().unwrap() = Some(global);
    });

    info!(wid, "handed new Surface to mpv");
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_ninelfx_hoshi_PlayerSurfaceView_nativeSurfaceDestroyed<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) {
    let Some(state) = app_state() else {
        warn!("nativeSurfaceDestroyed fired before android::attach() ran");
        return;
    };

    tauri::async_runtime::spawn(async move {
        if let Err(e) = state.playback.detach_surface().await {
            error!(?e, "failed to detach android surface from mpv");
        }
        *CURRENT_SURFACE.lock().unwrap() = None;
    });

    info!("released Surface from mpv");
}