use jni::objects::JObject;
use jni::JavaVM;
use tracing::{error, info};

/// reqwest and librqbit verify HTTPS through Android's trust store (rustls-platform-verifier),
/// which panics on the first handshake unless it was given the app context.
pub fn init() {
    let ctx = ndk_context::android_context();
    let result = (|| -> Result<(), jni::errors::Error> {
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };
        rustls_platform_verifier::android::init_with_env(&mut env, context)
    })();
    match result {
        Ok(()) => info!("TLS platform verifier initialized"),
        Err(e) => error!(error = %e, "Failed to initialize the TLS platform verifier"),
    }
}
