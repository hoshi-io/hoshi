fn main() {
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
        // Static libs from MPVKit, extracted by scripts/ios-mpv.sh. They are bundled into
        // the staticlib Xcode links; system deps come in via ios-plugin's HoshiSysLibs module.
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