use std::sync::OnceLock;

use jni::objects::{JClass, JObject};
use jni::JNIEnv;
use tracing::{error, info};

// Logging isn't set up yet when MainActivity calls in, so the result is logged from setup.
static RESULT: OnceLock<Result<(), String>> = OnceLock::new();

pub fn log_status() {
    match RESULT.get() {
        Some(Ok(())) => info!("TLS platform verifier initialized"),
        Some(Err(e)) => error!(error = %e, "Failed to initialize the TLS platform verifier"),
        None => error!("TLS platform verifier was never initialized (MainActivity.initTls not called)"),
    }
}

/// Called from `MainActivity.onCreate`, before any Rust code runs. reqwest and librqbit verify
/// HTTPS through Android's trust store (rustls-platform-verifier), which panics on the first
/// handshake unless it was given a context. Repeated calls are no-ops.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_ninelfx_hoshi_MainActivity_initTls<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    context: JObject<'local>,
) {
    let result = rustls_platform_verifier::android::init_with_env(&mut env, context).map_err(|e| e.to_string());
    let _ = RESULT.set(result);
}
