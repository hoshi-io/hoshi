//! UNVERIFIED written without a Mac. See the confidence notes at the
//! bottom of this file before debugging blind.

use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use hoshi_core::AppState;
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use libmpv2::Mpv;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::{NSRect, NSString};
use tauri::Manager;
use tracing::{debug, error, info, warn};

#[derive(Clone, Copy)]
struct SendPtr(*mut AnyObject);
unsafe impl Send for SendPtr {}

struct RenderState {
    ctx: RenderContext<'static>,
    _mpv: Arc<Mpv>,
}
unsafe impl Send for RenderState {}

fn opengl_framework_handle() -> *mut c_void {
    use std::sync::OnceLock;
    static HANDLE: OnceLock<usize> = OnceLock::new();
    let addr = *HANDLE.get_or_init(|| unsafe {
        let path = std::ffi::CString::new(
            "/System/Library/Frameworks/OpenGL.framework/OpenGL",
        )
            .unwrap();
        libc::dlopen(path.as_ptr(), libc::RTLD_NOW) as usize
    });
    addr as *mut c_void
}

fn get_proc_address(_ctx: &(), name: &str) -> *mut c_void {
    let Ok(cname) = std::ffi::CString::new(name) else {
        return std::ptr::null_mut();
    };
    let handle = opengl_framework_handle();
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { libc::dlsym(handle, cname.as_ptr()) }
}

pub fn attach(
    window: &tauri::WebviewWindow,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ready_notify = Arc::new(tokio::sync::Notify::new());
    window
        .app_handle()
        .manage(crate::player_surface::PlayerReadyNotify(ready_notify.clone()));

    let render_state: Arc<Mutex<Option<RenderState>>> = Arc::new(Mutex::new(None));

    let window_for_cb = window.clone();
    let state_for_cb = state.clone();
    let render_state_for_cb = render_state.clone();
    let ready_notify_for_cb = ready_notify.clone();

    window.with_webview(move |webview| unsafe {
        let webview_view: *mut AnyObject = webview.inner().cast();

        // Make the WKWebView see-through, same reasoning as Linux's
        let key = NSString::from_str("drawsBackground");
        let no_number: Retained<AnyObject> = msg_send![class!(NSNumber), numberWithBool: false];
        let _: () = msg_send![webview_view, setValue: &*no_number, forKey: &*key];

        let superview: *mut AnyObject = msg_send![webview_view, superview];
        if superview.is_null() {
            warn!("WKWebView has no superview yet; cannot insert GL view");
            return;
        }
        let bounds: NSRect = msg_send![superview, bounds];
        
        let attrs: [u32; 5] = [5, 73, 99, 0x3200, 0];
        let pixel_format: *mut AnyObject = msg_send![class!(NSOpenGLPixelFormat), alloc];
        let pixel_format: *mut AnyObject =
            msg_send![pixel_format, initWithAttributes: attrs.as_ptr()];
        if pixel_format.is_null() {
            error!("NSOpenGLPixelFormat initWithAttributes returned nil");
            return;
        }

        let gl_view: *mut AnyObject = msg_send![class!(NSOpenGLView), alloc];
        let gl_view: *mut AnyObject =
            msg_send![gl_view, initWithFrame: bounds, pixelFormat: pixel_format];
        if gl_view.is_null() {
            error!("NSOpenGLView initWithFrame:pixelFormat: returned nil");
            return;
        }

        let _: () = msg_send![gl_view, setAutoresizingMask: 18u64]; // Width|HeightSizable
        let _: () = msg_send![gl_view, setWantsBestResolutionOpenGLSurface: true];
        
        let _: () = msg_send![superview, addSubview: gl_view, positioned: -1isize, relativeTo: webview_view];

        let gl_context: *mut AnyObject = msg_send![gl_view, openGLContext];
        
        spawn_render_context_supervisor(
            window_for_cb.clone(),
            SendPtr(gl_view),
            SendPtr(gl_context),
            state_for_cb.clone(),
            render_state_for_cb.clone(),
            ready_notify_for_cb.clone(),
        );
    })?;

    Ok(())
}

fn spawn_render_context_supervisor(
    window: tauri::WebviewWindow,
    gl_view: SendPtr,
    gl_context: SendPtr,
    state: Arc<AppState>,
    render_state: Arc<Mutex<Option<RenderState>>>,
    ready_notify: Arc<tokio::sync::Notify>,
) {
    let shutdown_notify = state.playback.subscribe_shutdown();

    tauri::async_runtime::spawn(async move {
        loop {
            let mpv = loop {
                match state.playback.mpv_handle().await {
                    Ok(mpv) => break mpv,
                    Err(e) => {
                        debug!(error = ?e, "mpv not initialized yet; waiting for initialize_player");
                        ready_notify.notified().await;
                    }
                }
            };

            let window2 = window.clone();
            let render_state2 = render_state.clone();
            let _ = window.run_on_main_thread(move || {
                create_render_context(window2, gl_view, gl_context, mpv, render_state2);
            });

            shutdown_notify.notified().await;

            let render_state3 = render_state.clone();
            let _ = window.run_on_main_thread(move || unsafe {
                // Must be current for RenderContext::drop
                // (mpv_render_context_free) to tear down its GL objects.
                let _: () = msg_send![gl_context.0, makeCurrentContext];
                *render_state3.lock().unwrap() = None;
            });

            info!("render surface torn down after playback shutdown");
        }
    });
}

fn create_render_context(
    window: tauri::WebviewWindow,
    gl_view: SendPtr,
    gl_context: SendPtr,
    mpv: Arc<Mpv>,
    render_state: Arc<Mutex<Option<RenderState>>>,
) {
    unsafe {
        let _: () = msg_send![gl_context.0, makeCurrentContext];
    }

    let params = [
        RenderParam::ApiType(RenderParamApiType::OpenGl),
        RenderParam::InitParams(OpenGLInitParams { get_proc_address, ctx: () }),
    ];

    // SAFETY: same trick as Linux — `_mpv` in RenderState keeps this
    // alive; field order (ctx before _mpv) ensures render context drops first.
    let mpv_static: &'static Mpv = unsafe { &*Arc::as_ptr(&mpv) };

    match mpv_static.create_render_context(params) {
        Ok(mut ctx) => {
            info!("mpv render context created successfully");

            let render_state_for_cb = render_state.clone();
            ctx.set_update_callback({
                let window = window.clone();
                move || {
                    let render_state = render_state_for_cb.clone();
                    let _ = window.run_on_main_thread(move || {
                        render_frame(gl_view, gl_context, render_state);
                    });
                }
            });

            *render_state.lock().unwrap() = Some(RenderState { ctx, _mpv: mpv });
        }
        Err(e) => error!(error = ?e, "failed to create mpv render context"),
    }
}

fn render_frame(gl_view: SendPtr, gl_context: SendPtr, render_state: Arc<Mutex<Option<RenderState>>>) {
    let guard = render_state.lock().unwrap();
    let Some(rs) = guard.as_ref() else { return };

    unsafe {
        let _: () = msg_send![gl_context.0, makeCurrentContext];

        // convertRectToBacking: gives the Retina-correct pixel size in
        // one call, instead of manually multiplying by scale factor.
        let bounds: NSRect = msg_send![gl_view.0, bounds];
        let backing: NSRect = msg_send![gl_view.0, convertRectToBacking: bounds];
        let width = backing.size.width as i32;
        let height = backing.size.height as i32;

        if let Err(e) = rs.ctx.render::<()>(0, width, height, true) {
            error!(error = ?e, "mpv render failed");
        }

        let _: () = msg_send![gl_context.0, flushBuffer];
    }
}