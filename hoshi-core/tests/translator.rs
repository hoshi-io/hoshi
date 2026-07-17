use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use serde_json::json;
use tempfile::TempDir;
use tokio::time::timeout;

use hoshi_core::{
    build_app_state,
    paths::AppPaths,
    headless::noop_headless,
    logs::new_log_store,
};
use hoshi_core::extensions::types::{TachiyomiMarketplaceEntry, TachiyomiSource};

#[derive(Debug, Deserialize)]
struct RawTachiyomiMarketplaceEntry {
    pub name: String,
    pub pkg: String,
    pub apk: String,
    pub lang: String,
    pub version: String,
    pub nsfw: u8,
    pub sources: Vec<TachiyomiSource>,
}

fn repo_base_url(marketplace_url: &str) -> String {
    marketplace_url
        .rsplit_once('/')
        .map(|(base, _)| base.to_string())
        .unwrap_or_else(|| marketplace_url.trim_end_matches('/').to_string())
}

fn tachiyomi_icon_url(repo_url: &str, pkg: &str) -> String {
    format!("{}/icon/{}.png", repo_url.trim_end_matches('/'), pkg)
}

const STAGE_TIMEOUT: Duration = Duration::from_secs(30);
const GENERIC_SEARCH_QUERY: &str = "hero";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Working,
    Partial,
    Broken,
    NeedsHeadless,
}

impl Status {
    fn as_str(&self) -> &'static str {
        match self {
            Status::Working => "Working",
            Status::Partial => "Partial",
            Status::Broken => "Broken",
            Status::NeedsHeadless => "NeedsHeadless",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Install,
    Search,
    Metadata,
    Chapters,
    Pages,
    None,
}

impl Stage {
    fn as_str(&self) -> &'static str {
        match self {
            Stage::Install => "install",
            Stage::Search => "search",
            Stage::Metadata => "metadata",
            Stage::Chapters => "chapters",
            Stage::Pages => "pages",
            Stage::None => "none",
        }
    }
}

#[derive(Debug, Clone)]
struct ExtensionReport {
    pkg: String,
    name: String,
    lang: String,
    repo_url: String,
    status: Status,
    failed_stage: Stage,
    error_raw: String,
    error_normalized: String,
}

/// Strip known generic wrapper prefixes so we bucket errors by the actual
/// underlying cause (e.g. the JS exception text) rather than by the
/// wrapper wording that precedes *every* sandboxed error.
///
///   "Parse error: Failed to discover extension preferences: Internal error:
///    error.sandbox.js_exception: Error: ConcurrentHashMap is not defined"
///   -> "ConcurrentHashMap is not defined"
fn unwrap_error(raw: &str) -> &str {
    let mut s = raw;
    loop {
        let mut progressed = false;
        for marker in [
            "Parse error: Failed to discover extension preferences: ",
            "Internal error: ",
            "error.sandbox.js_exception: ",
            "error.sandbox.bad_json_response: ",
            "Error: ",
        ] {
            if let Some(rest) = s.strip_prefix(marker) {
                s = rest;
                progressed = true;
            }
        }
        if !progressed {
            // marker may appear mid-string in a nested "X: Y: <marker>Z" chain
            if let Some(pos) = s.find("error.sandbox.js_exception: ") {
                s = &s[pos + "error.sandbox.js_exception: ".len()..];
                progressed = true;
            } else if let Some(pos) = s.find("Internal error: ") {
                s = &s[pos + "Internal error: ".len()..];
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    s
}

/// Extract the bare identifier immediately preceding `suffix`, e.g.
/// extract_ident_before("ConcurrentHashMap is not defined", " is not defined", false)
///   -> Some("ConcurrentHashMap")
fn extract_ident_before<'a>(text: &'a str, suffix: &str, allow_dot: bool) -> Option<&'a str> {
    let pos = text.find(suffix)?;
    let before = &text[..pos];
    let ident_start = before
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$' || (allow_dot && c == '.')))
        .map(|i| i + 1)
        .unwrap_or(0);
    let ident = &before[ident_start..];
    if ident.is_empty() { None } else { Some(ident) }
}

fn normalize_error(raw: &str) -> String {
    if raw.contains("headless.fetch: not available") {
        return "headless not available".to_string();
    }

    // Peel back the generic wrapper text so we bucket by the *actual* cause
    // instead of by the boilerplate that precedes every sandboxed error.
    let inner = unwrap_error(raw);

    // Fixed, well-known error codes from CoreError -- map 1:1 to readable
    // text. These are engine/plumbing failures (network, manifest parsing,
    // sandbox init) as opposed to bugs in a specific extension's JS, so
    // it's worth keeping them distinct from the js_exception buckets below.
    const KNOWN_CODES: &[(&str, &str)] = &[
        ("error.extension.install_network_failed", "network error downloading extension/apk"),
        ("error.extension.invalid_manifest", "generated/parsed manifest invalid"),
        ("error.extension.unsupported_type", "unsupported extension type"),
        ("error.extension.invalid_script", "invalid extension script"),
        ("error.extension.script_missing", "extension script file missing on disk"),
        ("error.extension.not_found", "extension not found (installed but unregistered?)"),
        ("error.extension.not_tachiyomi", "not a tachiyomi-sourced extension"),
        ("error.sandbox.serialization_failed", "sandbox: failed to serialize args/settings/state"),
        ("error.sandbox.runtime_init_failed", "sandbox: QuickJS runtime init failed"),
        ("error.sandbox.thread_panicked", "sandbox: worker thread panicked"),
        ("error.sandbox.bad_json_response", "sandbox: extension returned non-JSON result"),
        ("error.content.invalid_extension_response", "invalid extension response"),
    ];
    for (code, label) in KNOWN_CODES {
        if inner.contains(code) {
            return label.to_string();
        }
    }

    // "<ident> is not defined" -- keep the identifier, so extensions missing
    // the *same* shim/global (e.g. ConcurrentHashMap) group together, and
    // extensions missing a *different* one don't get lumped together.
    if let Some(ident) = extract_ident_before(inner, " is not defined", false) {
        return format!("ReferenceError: {} is not defined", ident);
    }
    if inner.contains(" is not defined") {
        return "ReferenceError: <expr> is not defined".to_string();
    }

    if inner.contains("Cannot read property") || inner.contains("cannot read property")
        || inner.contains("Cannot read properties") || inner.contains("cannot read properties")
    {
        // Pull out the property name, e.g. "cannot read property 'isEmpty' of undefined"
        if let Some(start) = inner.find('\'') {
            if let Some(end) = inner[start + 1..].find('\'') {
                let prop = &inner[start + 1..start + 1 + end];
                return format!("cannot read property '{}' of undefined/null", prop);
            }
        }
        return "cannot read property of undefined/null".to_string();
    }

    if let Some(ident) = extract_ident_before(inner, " is not a function", true) {
        return format!("shim missing: {} is not a function", ident);
    }
    if inner.contains(" is not a function") {
        return "shim missing: <expr> is not a function".to_string();
    }

    if inner.contains("not found on") || inner.contains("does not exist on") {
        return "method not found on object".to_string();
    }

    if inner.contains("UnsupportedOperationException") {
        return "UnsupportedOperationException (shim missing)".to_string();
    }

    if inner.contains("Missing required field") {
        if let Some(pos) = inner.find("Missing required field in ") {
            let rest = &inner[pos + "Missing required field in ".len()..];
            let field = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if !field.is_empty() {
                return format!("missing required field in {}", field);
            }
        }
        return "missing required field".to_string();
    }

    if inner.contains("timeout") || inner.contains("timed out") {
        return "timeout".to_string();
    }

    if inner.contains("Network") || inner.contains("network") || inner.contains("dns error")
        || inner.contains("connection")
    {
        return "network error".to_string();
    }

    let trimmed = inner.trim();
    if trimmed.len() > 100 {
        format!("{}...", &trimmed[..100])
    } else {
        trimmed.to_string()
    }
}

const BAR_WIDTH: usize = 28;

fn status_color(status: Status) -> &'static str {
    match status {
        Status::Working => "\x1b[32m",       // green
        Status::Partial => "\x1b[33m",       // yellow
        Status::Broken => "\x1b[31m",        // red
        Status::NeedsHeadless => "\x1b[36m", // cyan
    }
}
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    } else {
        s.to_string()
    }
}

#[allow(clippy::too_many_arguments)]
fn print_progress(
    i: usize,
    total: usize,
    name: &str,
    pkg: &str,
    report: &ExtensionReport,
    n_working: usize,
    n_partial: usize,
    n_broken: usize,
    n_needs_headless: usize,
) {
    let filled = if total == 0 { 0 } else { (i * BAR_WIDTH) / total };
    let bar: String = "█".repeat(filled) + &"░".repeat(BAR_WIDTH - filled);
    let pct = if total == 0 { 100.0 } else { (i as f64 / total as f64) * 100.0 };

    let color = status_color(report.status);
    let status_tag = format!("{}{:<13}{}", color, report.status.as_str(), RESET);

    let detail = if report.failed_stage != Stage::None {
        format!(
            " {DIM}@ {}: {}{RESET}",
            report.failed_stage.as_str(),
            truncate(&report.error_normalized, 60)
        )
    } else {
        String::new()
    };

    // Clear the line, then print the current item on a rewritable line.
    print!(
        "\r\x1b[2K{BOLD}[{bar}] {i:>5}/{total:<5}{RESET} ({pct:>5.1}%)  {status_tag} {} {DIM}({}){RESET}{detail}",
        truncate(name, 28),
        truncate(pkg, 30),
    );


    if report.status != Status::Working {
        println!();
    }

    print!(
        "  {DIM}[\x1b[32mworking={}{RESET}{DIM} \x1b[33mpartial={}{RESET}{DIM} \x1b[31mbroken={}{RESET}{DIM} \x1b[36mneeds_headless={}{RESET}{DIM}]{RESET}",
        n_working, n_partial, n_broken, n_needs_headless
    );

    std::io::stdout().flush().ok();
}

#[tokio::test]
async fn tachiyomi_marketplace_pipeline() {
    let marketplace_url = std::env::var("MARKETPLACE_URL")
        .expect("set MARKETPLACE_URL env var to the tachiyomi marketplace index json url");

    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let tmp_path = tmp_dir.into_path();
    eprintln!("[tachiyomi_marketplace_pipeline] tmp dir: {}", tmp_path.display());
    let paths = AppPaths::from_base(tmp_path);
    let headless = noop_headless();
    let log_store = new_log_store();

    let state = build_app_state(paths, headless, log_store)
        .await
        .expect("failed to build AppState for test");

    // --- fetch marketplace ---
    let raw_entries: Vec<RawTachiyomiMarketplaceEntry> = state
        .http_client
        .get(&marketplace_url)
        .send()
        .await
        .expect("failed to fetch marketplace index")
        .json()
        .await
        .expect("failed to parse marketplace index json");

    let repo_url = repo_base_url(&marketplace_url);
    let entries: Vec<TachiyomiMarketplaceEntry> = raw_entries
        .into_iter()
        .map(|raw| {
            let icon_url = tachiyomi_icon_url(&repo_url, &raw.pkg);
            TachiyomiMarketplaceEntry {
                name: raw.name,
                pkg: raw.pkg,
                apk: raw.apk,
                lang: raw.lang,
                version: raw.version,
                nsfw: raw.nsfw,
                sources: raw.sources,
                repo_url: repo_url.clone(),
                icon_url: Some(icon_url),
            }
        })
        .collect();

    eprintln!("Loaded {} marketplace entries from repo: {}", entries.len(), repo_url);

    let mut reports: Vec<ExtensionReport> = Vec::with_capacity(entries.len());

    let csv_path = "tachiyomi_report.csv";
    let mut csv_file = File::create(csv_path).expect("failed to create report.csv");
    writeln!(
        csv_file,
        "pkg,name,lang,status,failed_stage,error_normalized,error_raw_summary,has_full_trace"
    )
        .unwrap();

    let errors_log_path = "tachiyomi_errors.log";
    let mut errors_log = File::create(errors_log_path).expect("failed to create errors.log");

    let total = entries.len();

    let mut n_working = 0usize;
    let mut n_partial = 0usize;
    let mut n_broken = 0usize;
    let mut n_needs_headless = 0usize;

    for (i, entry) in entries.into_iter().enumerate() {
        let pkg_for_log = entry.pkg.clone();
        let name_for_log = entry.name.clone();

        let report = run_pipeline_for_entry(&state, entry).await;

        match report.status {
            Status::Working => n_working += 1,
            Status::Partial => n_partial += 1,
            Status::Broken => n_broken += 1,
            Status::NeedsHeadless => n_needs_headless += 1,
        }

        print_progress(
            i + 1,
            total,
            &name_for_log,
            &pkg_for_log,
            &report,
            n_working,
            n_partial,
            n_broken,
            n_needs_headless,
        );

        fn csv_field(s: &str) -> String {
            format!("\"{}\"", s.replace('"', "\"\""))
        }

        let error_raw_first_line = report.error_raw.lines().next().unwrap_or("");
        let has_full_trace = report.error_raw.lines().count() > 1;

        if has_full_trace {
            writeln!(
                errors_log,
                "=== {} ({}) @ {} ===\n{}\n",
                report.pkg,
                report.name,
                report.failed_stage.as_str(),
                report.error_raw
            )
                .unwrap();
            errors_log.flush().unwrap();
        }

        writeln!(
            csv_file,
            "{},{},{},{},{},{},{},{}",
            csv_field(&report.pkg),
            csv_field(&report.name),
            csv_field(&report.lang),
            report.status.as_str(),
            report.failed_stage.as_str(),
            csv_field(&report.error_normalized),
            csv_field(error_raw_first_line),
            has_full_trace,
        )
            .unwrap();
        csv_file.flush().unwrap();

        reports.push(report);
    }

    println!(
        "\n=== FINAL: {} total | working={} partial={} broken={} needs_headless={} ===",
        total, n_working, n_partial, n_broken, n_needs_headless
    );

    write_markdown_summary(&reports, "tachiyomi_report.md", &repo_url);
}

async fn run_pipeline_for_entry(
    state: &Arc<hoshi_core::AppState>,
    entry: TachiyomiMarketplaceEntry,
) -> ExtensionReport {
    let pkg = entry.pkg.clone();
    let name = entry.name.clone();
    let lang = entry.lang.clone();
    let repo_url = entry.repo_url.clone();

    let download_url = format!(
        "{}/apk/{}",
        entry.repo_url.trim_end_matches('/'),
        entry.apk
    );

    macro_rules! fail {
        ($stage:expr, $raw:expr) => {{
            let raw = $raw;
            let normalized = normalize_error(&raw);
            let status = if normalized == "headless not available" {
                Status::NeedsHeadless
            } else if $stage == Stage::Install || $stage == Stage::Search {
                Status::Broken
            } else {
                Status::Partial
            };
            return ExtensionReport {
                pkg,
                name,
                lang,
                repo_url,
                status,
                failed_stage: $stage,
                error_raw: raw,
                error_normalized: normalized,
            };
        }};
    }

    // --- install ---
    let extension = {
        let mut mgr = state.extension_manager.write().await;
        match timeout(
            STAGE_TIMEOUT,
            mgr.install_tachiyomi_extension(state, &download_url, entry),
        )
            .await
        {
            Ok(Ok(ext)) => ext,
            Ok(Err(e)) => fail!(Stage::Install, e.to_string()),
            Err(_) => fail!(Stage::Install, "timeout".to_string()),
        }
    };

    let ext_id = extension.id.clone();

    // --- search ---
    let search_results = {
        let mgr = state.extension_manager.read().await;

        let empty_attempt = timeout(
            STAGE_TIMEOUT,
            mgr.search(&ext_id, "", json!({}), 1),
        )
            .await;

        match empty_attempt {
            Ok(Ok(results)) if !results.is_empty() => Ok(results),
            _ => {
                // fall back to generic query
                timeout(
                    STAGE_TIMEOUT,
                    mgr.search(&ext_id, GENERIC_SEARCH_QUERY, json!({}), 1),
                )
                    .await
                    .unwrap_or_else(|_| Err(hoshi_core::error::CoreError::Internal(
                        "timeout".into(),
                    )))
            }
        }
    };

    let search_results = match search_results {
        Ok(results) if !results.is_empty() => results,
        Ok(_) => fail!(Stage::Search, "empty results (latestManga and searchManga both returned nothing)".to_string()),
        Err(e) => fail!(Stage::Search, e.to_string()),
    };

    let content_id = search_results[0].id.clone();

    // --- metadata ---
    {
        let mgr = state.extension_manager.read().await;
        match timeout(STAGE_TIMEOUT, mgr.get_metadata(&ext_id, &content_id)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => fail!(Stage::Metadata, e.to_string()),
            Err(_) => fail!(Stage::Metadata, "timeout".to_string()),
        }
    }

    // --- chapters ---
    let chapters = {
        let mgr = state.extension_manager.read().await;
        match timeout(STAGE_TIMEOUT, mgr.find_chapters(&ext_id, &content_id)).await {
            Ok(Ok(chapters)) if !chapters.is_empty() => chapters,
            Ok(Ok(_)) => fail!(Stage::Chapters, "zero chapters returned".to_string()),
            Ok(Err(e)) => fail!(Stage::Chapters, e.to_string()),
            Err(_) => fail!(Stage::Chapters, "timeout".to_string()),
        }
    };

    let chapter_id = chapters[0].id.clone();

    // --- pages ---
    {
        let mgr = state.extension_manager.read().await;
        match timeout(STAGE_TIMEOUT, mgr.find_manga_pages(&ext_id, &chapter_id)).await {
            Ok(Ok(pages)) if !pages.is_empty() => {}
            Ok(Ok(_)) => fail!(Stage::Pages, "zero pages returned".to_string()),
            Ok(Err(e)) => fail!(Stage::Pages, e.to_string()),
            Err(_) => fail!(Stage::Pages, "timeout".to_string()),
        }
    }

    ExtensionReport {
        pkg,
        name,
        lang,
        repo_url,
        status: Status::Working,
        failed_stage: Stage::None,
        error_raw: String::new(),
        error_normalized: String::new(),
    }
}

fn write_markdown_summary(reports: &[ExtensionReport], path: &str, repo_url: &str) {
    let total = reports.len();
    let count = |s: Status| reports.iter().filter(|r| r.status == s).count();

    let working = count(Status::Working);
    let partial = count(Status::Partial);
    let broken = count(Status::Broken);
    let needs_headless = count(Status::NeedsHeadless);

    let pct = |n: usize| {
        if total == 0 {
            0.0
        } else {
            (n as f64 / total as f64) * 100.0
        }
    };

    let mut out = String::new();
    out.push_str("# Tachiyomi Marketplace Pipeline Report\n\n");
    out.push_str(&format!("Repo: `{}`\n\n", repo_url));
    out.push_str(&format!("Total extensions tested: **{}**\n\n", total));
    out.push_str("## Summary\n\n");
    out.push_str("| Status | Count | % |\n|---|---|---|\n");
    out.push_str(&format!("| Working | {} | {:.1}% |\n", working, pct(working)));
    out.push_str(&format!("| Partial | {} | {:.1}% |\n", partial, pct(partial)));
    out.push_str(&format!("| Broken | {} | {:.1}% |\n", broken, pct(broken)));
    out.push_str(&format!(
        "| Needs Headless | {} | {:.1}% |\n\n",
        needs_headless,
        pct(needs_headless)
    ));

    // Breakdown by failed stage (excludes Working entries, which have Stage::None)
    out.push_str("## Failures by stage\n\n");
    out.push_str("| Stage | Count |\n|---|---|\n");
    for stage in [Stage::Install, Stage::Search, Stage::Metadata, Stage::Chapters, Stage::Pages] {
        let n = reports.iter().filter(|r| r.failed_stage == stage).count();
        if n > 0 {
            out.push_str(&format!("| {} | {} |\n", stage.as_str(), n));
        }
    }
    out.push('\n');

    // Top normalized error buckets
    let mut error_counts: HashMap<&str, (usize, &ExtensionReport)> = HashMap::new();
    for r in reports {
        if r.error_normalized.is_empty() {
            continue;
        }
        error_counts
            .entry(r.error_normalized.as_str())
            .and_modify(|(n, _)| *n += 1)
            .or_insert((1, r));
    }
    let mut error_vec: Vec<_> = error_counts.into_iter().collect();
    error_vec.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    out.push_str("## Top errors\n\n");
    out.push_str("| Error (normalized) | Count | Example (pkg) | Example (raw) |\n|---|---|---|---|\n");
    for (err, (n, example)) in error_vec.into_iter().take(15) {
        let raw_short = if example.error_raw.len() > 100 {
            format!("{}...", &example.error_raw[..100])
        } else {
            example.error_raw.clone()
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            err.replace('|', "\\|"),
            n,
            example.pkg,
            raw_short.replace('|', "\\|")
        ));
    }
    out.push_str("\nFull per-extension data (including every raw error) is in `tachiyomi_report.csv`.\n");

    std::fs::write(path, out).expect("failed to write markdown report");
}