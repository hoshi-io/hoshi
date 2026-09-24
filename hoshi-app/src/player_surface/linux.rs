//! Linux GL/webview compositing surface using GTK Overlay.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk::prelude::*;
use hoshi_core::AppState;
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use libmpv2::Mpv;
use tauri::Manager;
use tracing::{debug, error, info, instrument, warn};

fn get_proc_address(_ctx: &(), name: &str) -> *mut std::ffi::c_void {
    let Ok(cname) = std::ffi::CString::new(name) else {
        return std::ptr::null_mut();
    };
    unsafe { libc::dlsym(libc::RTLD_DEFAULT, cname.as_ptr()) }
}

/// Field order enforces `ctx` drops before `_mpv`, per mpv's requirement
/// that the render context be freed before the core handle.
struct RenderState {
    ctx: RenderContext<'static>,
    _mpv: Arc<Mpv>,
}

#[instrument(skip(window, state))]
pub fn attach(
    window: &tauri::WebviewWindow,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let gtk_window = window.gtk_window()?;

    let webview_container = gtk_window
        .child()
        .ok_or("expected wry's webview container to already be attached to the window")?;
    let webview_container: gtk::Container = webview_container
        .downcast()
        .map_err(|_| "expected the window's child to be a GTK container wrapping the webview")?;

    let existing_child = webview_container
        .children()
        .into_iter()
        .next()
        .ok_or("expected the webview container to have the webview as its child")?;
    info!(
        webview_container_type = %webview_container.type_().name(),
        webview_child_type = %existing_child.type_().name(),
        webview_container_children = webview_container.children().len(),
        "extracted webview widget from its original container"
    );
    webview_container.remove(&existing_child);
    gtk_window.remove(&webview_container);

    existing_child.set_hexpand(true);
    existing_child.set_vexpand(true);

    let overlay = gtk::Overlay::new();
    let gl_area = gtk::GLArea::new();
    gl_area.set_auto_render(false);
    gl_area.set_hexpand(true);
    gl_area.set_vexpand(true);

    let render_state: Rc<RefCell<Option<RenderState>>> = Rc::new(RefCell::new(None));
    let ready_notify = std::sync::Arc::new(tokio::sync::Notify::new());

    window
        .app_handle()
        .manage(crate::player_surface::PlayerReadyNotify(ready_notify.clone()));

    gl_area.connect_realize({
        let render_state = render_state.clone();
        let state = state.clone();
        let ready_notify = ready_notify.clone();
        move |area| {
            spawn_render_context_supervisor(area.clone(), state.clone(), render_state.clone(), ready_notify.clone());
        }
    });

    gl_area.connect_render({
        let render_state = render_state.clone();
        let render_call_count = Rc::new(std::cell::Cell::new(0u32));
        move |area, _gl_ctx| {
            let call_count = render_call_count.get() + 1;
            render_call_count.set(call_count);

            let guard = render_state.borrow();
            let Some(rs) = guard.as_ref() else {
                if call_count <= 5 || call_count % 60 == 0 {
                    debug!(call_count, "connect_render fired but no RenderContext yet");
                }
                // GtkGLArea doesn't clear its backing buffer for us, so
                // without this the last mpv frame stays on screen forever.
                clear_gl_area();
                return glib::Propagation::Stop;
            };

            let scale = area.scale_factor();
            let width = area.allocated_width() * scale;
            let height = area.allocated_height() * scale;

            let mut current_fbo: i32 = 0;
            unsafe {
                let get_integerv_ptr = get_proc_address(&(), "glGetIntegerv");
                if !get_integerv_ptr.is_null() {
                    let gl_get_integerv: extern "C" fn(u32, *mut i32) = std::mem::transmute(get_integerv_ptr);
                    gl_get_integerv(0x8CA6, &mut current_fbo); // GL_FRAMEBUFFER_BINDING
                }
            }

            let render_result = rs.ctx.render::<()>(current_fbo, width, height, true);

            if call_count <= 5 || call_count % 60 == 0 {
                debug!(call_count, width, height, ok = render_result.is_ok(), "mpv render() called");
            }
            if let Err(e) = render_result {
                error!(error = ?e, "mpv render failed");
            }

            glib::Propagation::Stop
        }
    });

    overlay.add(&gl_area);
    overlay.add_overlay(&existing_child);
    overlay.set_overlay_pass_through(&existing_child, false);

    gtk_window.add(&overlay);
    overlay.show_all();
    info!(
        window_child_type = %gtk_window.child().map(|w| w.type_().name().to_string()).unwrap_or_default(),
        "window restructured: Overlay is now the direct child"
    );

    if let Err(e) = window.with_webview(|webview| {
        use webkit2gtk::WebViewExt;
        webview
            .inner()
            .set_background_color(&gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
    }) {
        warn!(error = ?e, "failed to set webview transparent background");
    }

    Ok(())
}

/// Clears the current framebuffer to transparent black. Used whenever
/// `render` fires with no mpv context installed.
fn clear_gl_area() {
    unsafe {
        let clear_color_ptr = get_proc_address(&(), "glClearColor");
        let clear_ptr = get_proc_address(&(), "glClear");
        if clear_color_ptr.is_null() || clear_ptr.is_null() {
            warn!("could not resolve glClearColor/glClear to clear stale mpv frame");
            return;
        }
        let gl_clear_color: extern "C" fn(f32, f32, f32, f32) = std::mem::transmute(clear_color_ptr);
        let gl_clear: extern "C" fn(u32) = std::mem::transmute(clear_ptr);
        gl_clear_color(0.0, 0.0, 0.0, 0.0);
        gl_clear(0x4000); // GL_COLOR_BUFFER_BIT
    }
}

/// Runs for the GLArea's lifetime: waits for a live mpv session, builds a
/// render context for it, waits for shutdown, tears it down, and repeats —
/// so shutdown/reinitialize cycles rebuild the surface correctly.
///
/// Lives on the GTK main context (`spawn_local`, not
/// `tauri::async_runtime::spawn`): it holds `Rc`/GTK types across `.await`
/// points, which aren't `Send`.
fn spawn_render_context_supervisor(
    area: gtk::GLArea,
    state: Arc<AppState>,
    render_state: Rc<RefCell<Option<RenderState>>>,
    ready_notify: Arc<tokio::sync::Notify>,
) {
    let shutdown_notify = state.playback.subscribe_shutdown();

    glib::MainContext::default().spawn_local(async move {
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

            create_render_context(area.clone(), mpv, render_state.clone()).await;

            shutdown_notify.notified().await;

            // Must be current for RenderContext::drop (mpv_render_context_free)
            // to tear down its GL objects safely.
            area.make_current();
            if let Some(err) = area.error() {
                error!(error = %err, "gl_area reports an error before render-context teardown");
            }

            *render_state.borrow_mut() = None;
            info!("render surface torn down after playback shutdown");
            area.queue_render();
        }
    });
}

async fn create_render_context(
    area: gtk::GLArea,
    mpv: Arc<Mpv>,
    render_state: Rc<RefCell<Option<RenderState>>>,
) {
    // `connect_realize` fires once per widget, so on the 2nd+ session this
    // is what actually makes the context current.
    area.make_current();
    if let Some(err) = area.error() {
        error!(error = %err, "gl_area reports an error before render-context creation");
    }

    let params = [
        RenderParam::ApiType(RenderParamApiType::OpenGl),
        RenderParam::InitParams(OpenGLInitParams {
            get_proc_address,
            ctx: (),
        }),
    ];

    // SAFETY: `_mpv` in RenderState keeps this alive; field order (ctx
    // before _mpv) ensures the render context always drops first.
    let mpv_static: &'static Mpv = unsafe { &*Arc::as_ptr(&mpv) };

    match mpv_static.create_render_context(params) {
        Ok(mut ctx) => {
            info!("mpv render context created successfully");

            let area_weak: glib::SendWeakRef<gtk::GLArea> = area.downgrade().into();
            let update_call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

            ctx.set_update_callback(move || {
                let n = update_call_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                if n <= 5 || n % 60 == 0 {
                    debug!(call_count = n, "mpv update callback fired");
                }
                let area_weak = area_weak.clone();
                glib::source::idle_add(move || {
                    match area_weak.upgrade() {
                        Some(area) => area.queue_render(),
                        None => warn!("update callback fired but GLArea is gone"),
                    }
                    glib::ControlFlow::Break
                });
            });

            *render_state.borrow_mut() = Some(RenderState { ctx, _mpv: mpv });
            area.queue_render();
        }
        Err(e) => error!(error = ?e, "failed to create mpv render context"),
    }
}