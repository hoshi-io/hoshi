use std::io::{Cursor, Read};
use zip::ZipArchive;

use hoshi_core::extensions::tachiyomi_loader::{
    extract_dex, inspect_apk_reader, walk_source,
};
use hoshi_core::extensions::tachiyomi_loader::translator::{self, resolver};
use hoshi_core::extensions::tachiyomi_loader::translator::resolver::pool::Pool;

const DEFAULT_URL: &str =
    "https://github.com/keiyoushi/extensions/raw/refs/heads/repo/apk/tachiyomi-all.mitaku-v1.4.4.apk";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_URL.to_string());

    // ── 1. Download ───────────────────────────────────────────────────────────
    eprintln!("[1/4] Downloading {url} ...");
    let mut bytes = Vec::new();
    ureq::get(&url)
        .call()?
        .into_reader()
        .read_to_end(&mut bytes)?;
    eprintln!("      {} KB received", bytes.len() / 1024);

    // ── 2. Inspect APK metadata ───────────────────────────────────────────────
    eprintln!("[2/4] Inspecting APK ...");
    let meta = inspect_apk_reader(Cursor::new(&bytes))?;
    eprintln!("      name={:?}  lang={:?}  package={:?}  v={}",
              meta.name, meta.lang, meta.package, meta.version_name);

    // ── 3. Extract DEX + walk ─────────────────────────────────────────────────
    eprintln!("[3/4] Extracting + walking DEX ...");
    let mut zip = ZipArchive::new(Cursor::new(&bytes))?;
    let extracted = extract_dex(&mut zip, &meta)?;
    let pool = Pool::build(&extracted.dex_files);

    let walked = walk_source(&extracted, &meta)?;

    // ── 4. Translate + resolve ────────────────────────────────────────────────
    eprintln!("[4/4] Translating ...");
    let translated = translator::translate(&walked, &meta, &pool)?;

    if translated.has_warnings() {
        eprintln!("      {} warning(s):", translated.warnings.len());
        for w in &translated.warnings {
            eprintln!("        ⚠  {w}");
        }
    }

    let final_js = resolver::resolve::resolve(&translated.js, &pool);

    let out_path = "/tmp/test_apk_out.js";
    std::fs::write(out_path, &final_js)?;
    eprintln!("\nWritten to {out_path}  ({} bytes)", final_js.len());

    let lines: Vec<&str> = final_js.lines().collect();
    for (i, line) in lines.iter().enumerate().take(120) {
        println!("{:>4}  {line}", i + 1);
    }
    if lines.len() > 120 {
        println!("      ... (+{} more lines, see {out_path})", lines.len() - 120);
    }

    Ok(())
}