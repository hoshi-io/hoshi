fn git(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Commit + target for the startup diagnostics. `-dirty` covers this crate only (hoshi-core
/// reports its own; the frontend is bundled separately).
fn version_info() {
    let commit = match git(&["rev-parse", "--short=10", "HEAD"]) {
        Some(hash) => {
            let dirty = git(&["status", "--porcelain", "--untracked-files=no", "--", "."]).is_some_and(|s| !s.is_empty());
            if dirty { format!("{hash}-dirty") } else { hash }
        }
        None => "unknown".into(),
    };
    println!("cargo:rustc-env=HOSHI_APP_COMMIT={commit}");
    println!("cargo:rustc-env=HOSHI_APP_TARGET={}", std::env::var("TARGET").unwrap());

    // Rerun on a new commit/checkout, and on edits (which recompile the crate anyway).
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        let mut watch = vec![format!("{git_dir}/HEAD"), format!("{git_dir}/packed-refs")];
        if let Some(head_ref) = git(&["symbolic-ref", "-q", "HEAD"]) {
            watch.push(format!("{git_dir}/{head_ref}"));
        }
        for path in watch.iter().filter(|p| std::path::Path::new(p).exists()) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    for path in ["build.rs", "Cargo.toml", "src"] {
        println!("cargo:rerun-if-changed={path}");
    }
}

fn main() {
    version_info();
    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mpv_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("mpv-windows");
        println!("cargo:rustc-link-search=native={}", mpv_dir.display());
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        let abi = match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "aarch64" => "arm64-v8a",
            "arm"     => "armeabi-v7a",
            "x86"     => "x86",
            "x86_64"  => "x86_64",
            other => panic!("unsupported android arch: {other}"),
        };
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let lib_dir = std::path::Path::new(&manifest_dir).join("mpv-android").join(abi);
        println!("cargo:rustc-link-search=native={}", lib_dir.display());

        // Needed explicitly: av_jni_set_java_vm lives in libavcodec, but
        // nothing in our own Rust code calls into it except the raw
        // `extern "C"` decl in android.rs, so the linker won't pull this
        // .so in on its own — it has to be named directly.
        println!("cargo:rustc-link-lib=dylib=avcodec");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("ios") {
        #[cfg(target_os = "macos")]
        build_ios_plugins();

        // Static libs from MPVKit, extracted by scripts/ios-mpv.sh. They are bundled into
        // the staticlib Xcode links; system deps come in via HoshiPlugins' HoshiSysLibs module.
        let target = std::env::var("TARGET").unwrap();
        let slice = if target.ends_with("-sim") || target.starts_with("x86_64") { "simulator" } else { "device" };
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let lib_dir = std::path::Path::new(&manifest_dir).join("mpv-ios").join(slice);
        println!("cargo:rerun-if-changed={}", lib_dir.display());

        let libs: Vec<String> = std::fs::read_dir(&lib_dir)
            .unwrap_or_else(|_| panic!("{} missing, run scripts/ios-mpv.sh first", lib_dir.display()))
            .filter_map(|e| {
                let name = e.ok()?.file_name().into_string().ok()?;
                Some(name.strip_prefix("lib")?.strip_suffix(".a")?.to_owned())
            })
            .collect();
        assert!(libs.iter().any(|l| l == "mpv"), "libmpv.a not found in {}", lib_dir.display());

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        for lib in libs {
            println!("cargo:rustc-link-lib=static={lib}");
        }

        // Only needed for the throwaway cdylib cargo also links; the staticlib gets these
        // through autolinking at Xcode's link step.
        for lib in ["z", "bz2", "iconv", "expat", "resolv", "xml2", "c++"] {
            println!("cargo:rustc-link-lib=dylib={lib}");
        }
        for fw in [
            "AVFoundation", "CoreAudio", "AudioToolbox", "CoreVideo", "CoreMedia", "CoreText",
            "VideoToolbox", "Metal", "IOSurface", "QuartzCore", "MediaPlayer", "UIKit",
            "CoreGraphics", "Foundation", "Security",
        ] {
            println!("cargo:rustc-link-lib=framework={fw}");
        }
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let mpv_prefix = std::process::Command::new("brew")
            .args(["--prefix", "mpv"])
            .output()
            .expect("failed to find Homebrew mpv");

        let prefix = String::from_utf8(mpv_prefix.stdout)
            .expect("invalid brew output")
            .trim()
            .to_owned();

        println!("cargo:rustc-link-search=native={prefix}/lib");
        println!("cargo:rustc-link-lib=dylib=mpv");
    }
}
/// Compiles the Swift plugins in gen/apple/HoshiPlugins (what a Tauri plugin crate's build
/// does for its ios/ folder). They import the Tauri Swift package, so it is copied next to them.
#[cfg(target_os = "macos")]
fn build_ios_plugins() {
    use std::path::{Path, PathBuf};

    fn copy_dir(src: &Path, dst: &Path) {
        std::fs::create_dir_all(dst).unwrap();
        for entry in std::fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            if [".build", "Package.resolved", "Tests"].iter().any(|skip| name == *skip) {
                continue;
            }
            let (from, to) = (entry.path(), dst.join(&name));
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&from, &to);
            } else {
                std::fs::copy(&from, &to).unwrap();
            }
        }
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let apple_dir = manifest_dir.join("gen").join("apple");
    let tauri_api = PathBuf::from(
        std::env::var("DEP_TAURI_IOS_LIBRARY_PATH").expect("tauri did not export its iOS library path"),
    );

    let dest = apple_dir.join(".tauri").join("tauri-api");
    let _ = std::fs::remove_dir_all(&dest);
    copy_dir(&tauri_api, &dest);

    let plugins = apple_dir.join("HoshiPlugins");
    println!("cargo:rerun-if-changed={}", plugins.join("Sources").display());
    println!("cargo:rerun-if-changed={}", plugins.join("SysLibs").display());
    println!("cargo:rerun-if-changed={}", plugins.join("Package.swift").display());
    tauri_utils::build::link_apple_library("hoshi-plugins", plugins);
}
