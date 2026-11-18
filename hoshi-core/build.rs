//! Bakes the git commit into the binary for the startup diagnostics.
use std::path::Path;
use std::process::Command;

fn git(dir: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(dir).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let commit = match git(&dir, &["rev-parse", "--short=10", "HEAD"]) {
        Some(hash) => {
            // Uncommitted changes to this crate only (the app and frontend have their own).
            let dirty = git(&dir, &["status", "--porcelain", "--untracked-files=no", "--", "."])
                .is_some_and(|s| !s.is_empty());
            if dirty { format!("{hash}-dirty") } else { hash }
        }
        None => "unknown".into(),
    };
    println!("cargo:rustc-env=HOSHI_CORE_COMMIT={commit}");

    // Rerun on a new commit/checkout, and on edits (which recompile the crate anyway).
    if let Some(git_dir) = git(&dir, &["rev-parse", "--absolute-git-dir"]) {
        let mut watch = vec![format!("{git_dir}/HEAD"), format!("{git_dir}/packed-refs")];
        if let Some(head_ref) = git(&dir, &["symbolic-ref", "-q", "HEAD"]) {
            watch.push(format!("{git_dir}/{head_ref}"));
        }
        for path in watch.iter().filter(|p| Path::new(p).exists()) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    for path in ["build.rs", "Cargo.toml", "src", "migrations"] {
        println!("cargo:rerun-if-changed={path}");
    }
}
