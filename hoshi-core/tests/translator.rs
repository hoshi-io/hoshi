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

fn normalize_error(raw: &str) -> String {
    if raw.contains("headless.fetch: not available") {
        return "headless not available".to_string();
    }

    // "<ident> is not defined"
    if let Some(pos) = raw.find(" is not defined") {
        let before = &raw[..pos];
        let ident_start = before
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
            .map(|i| i + 1)
            .unwrap_or(0);
        let ident = &before[ident_start..];
        if !ident.is_empty() {
            return "<ident> is not defined".to_string();
        }
        return "<expr> is not defined".to_string();
    }

    if raw.contains("Cannot read property") || raw.contains("Cannot read properties") {
        return "cannot read property of undefined/null".to_string();
    }

    if let Some(pos) = raw.find(" is not a function") {
        let before = &raw[..pos];
        let ident_start = before
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$' || c == '.'))
            .map(|i| i + 1)
            .unwrap_or(0);
        let ident = &before[ident_start..];
        if !ident.is_empty() {
            return format!("shim missing: {} is not a function", ident);
        }
        return "<expr> is not a function".to_string();
    }

    if raw.contains("not found on") || raw.contains("does not exist on") {
        return "method not found on object".to_string();
    }

    if raw.contains("timeout") || raw.contains("timed out") {
        return "timeout".to_string();
    }

    if raw.contains("error.sandbox.bad_json_response") {
        return "bad json response from sandbox".to_string();
    }

    if raw.contains("Network") || raw.contains("network") || raw.contains("dns error")
        || raw.contains("connection")
    {
        return "network error".to_string();
    }

    let trimmed = raw.trim();
    if trimmed.len() > 80 {
        format!("{}...", &trimmed[..80])
    } else {
        trimmed.to_string()
    }
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

        print!(
            "[{}/{}] {} ({}) -> {}{}  || running totals: working={} partial={} broken={} needs_headless={}\r\n",
            i + 1,
            total,
            name_for_log,
            pkg_for_log,
            report.status.as_str(),
            if report.failed_stage != Stage::None {
                format!(" @ {}: {}", report.failed_stage.as_str(), report.error_normalized)
            } else {
                String::new()
            },
            n_working,
            n_partial,
            n_broken,
            n_needs_headless,
        );
        std::io::stdout().flush().ok();

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