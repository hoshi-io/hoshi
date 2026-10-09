use std::cmp::Ordering;
use std::path::Path;

use crate::torrent::parser::matcher::TargetDescriptor;
use crate::torrent::parser::parser::{parse_release_title, season_from_title, ParsedRelease};

const VIDEO_EXT: &[&str] = &[
    "mkv", "mp4", "m4v", "avi", "webm", "mov", "wmv", "flv", "ts", "m2ts",
];

#[derive(Debug, Clone)]
pub struct TorrentFile {
    /// Index in the torrent's file list (what `torrent.stream()` takes).
    pub index: usize,
    /// Relative path inside the torrent, '/'-separated.
    pub path: String,
    pub len: u64,
}

fn is_video(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| VIDEO_EXT.iter().any(|v| e.eq_ignore_ascii_case(v)))
}

/// Parses the file name; the folders only contribute a season marker.
fn parse_file(path: &str) -> (ParsedRelease, Option<u32>) {
    let (dirs, name) = match path.rsplit_once('/') {
        Some((d, n)) => (d, n),
        None => ("", path),
    };

    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);

    // "05.mkv" / "05v2.mkv": the release parser needs a title in front of the number.
    let digits = stem.trim_end_matches(|c: char| c.is_ascii_digit());
    let bare = !stem.is_empty()
        && (digits.is_empty()
        || (digits.eq_ignore_ascii_case("v") && stem.len() > 1 && stem.as_bytes()[0].is_ascii_digit()));
    let parsed = if bare {
        let (n, v) = match stem.to_ascii_lowercase().split_once('v') {
            Some((n, v)) => (n.to_owned(), v.parse().unwrap_or(1)),
            None => (stem.to_owned(), 1),
        };
        ParsedRelease {
            episode: n.parse().ok(),
            version: v,
            ..Default::default()
        }
    } else {
        parse_release_title(name)
    };

    // closest folder with a season marker wins
    let dir_season = dirs.rsplit('/').find_map(season_from_title);
    (parsed, dir_season)
}

fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut n = 0u64;
                    while let Some(d) = it.peek().and_then(|c| c.to_digit(10)) {
                        n = n.saturating_mul(10).saturating_add(d as u64);
                        it.next();
                    }
                    n
                };
                let (na, nb) = (take(&mut a), take(&mut b));
                if na != nb {
                    return na.cmp(&nb);
                }
            }
            (Some(x), Some(y)) => {
                let o = x.to_ascii_lowercase().cmp(&y.to_ascii_lowercase());
                if o != Ordering::Equal {
                    return o;
                }
                a.next();
                b.next();
            }
        }
    }
}

/// Returns the `index` of the file to play.
///
/// * no target, or a single-video torrent, or a movie: the largest video file
///   (what happened before batches were supported)
/// * otherwise the file whose parsed episode matches the target
/// * if no file name carries an episode number at all: position in natural
///   order, but only when the file count equals the entry's episode count
///
/// `None` means "this torrent does not contain that episode".
pub fn select_file(files: &[TorrentFile], target: Option<&TargetDescriptor>) -> Option<usize> {
    let videos: Vec<&TorrentFile> = files.iter().filter(|f| is_video(&f.path)).collect();
    if videos.is_empty() {
        return files.iter().max_by_key(|f| f.len).map(|f| f.index);
    }
    let largest = || videos.iter().max_by_key(|f| f.len).map(|f| f.index);

    let Some(target) = target else { return largest() };
    if videos.len() == 1 || target.total_episodes == Some(1) {
        return largest();
    }

    let parsed: Vec<(&TorrentFile, ParsedRelease, Option<u32>)> = videos
        .iter()
        .map(|f| {
            let (p, ds) = parse_file(&f.path);
            (*f, p, ds)
        })
        .collect();

    // Folder seasons are a hint: a pack folder like "Show Season 1-2" must not
    // veto the right file, so retry without them.
    let pick = |use_dirs: bool| {
        parsed
            .iter()
            .filter_map(|(f, p, ds)| {
                target
                    .evaluate_file(p, if use_dirs { *ds } else { None })
                    .map(|score| (f, p, score))
            })
            .max_by(|a, b| {
                a.2.partial_cmp(&b.2)
                    .unwrap_or(Ordering::Equal)
                    .then(a.1.version.cmp(&b.1.version)) // v2 over v1
                    .then(a.0.len.cmp(&b.0.len))
            })
            .map(|(f, _, _)| f.index)
    };
    if let Some(i) = pick(true).or_else(|| pick(false)) {
        return Some(i);
    }

    let nothing_parsed = parsed
        .iter()
        .all(|(_, p, _)| p.episode.is_none() && p.episode_range.is_none());
    if nothing_parsed {
        let mut ordered: Vec<&TorrentFile> = parsed
            .iter()
            .filter(|(_, p, _)| !p.is_extra || target.allow_extras)
            .map(|(f, _, _)| *f)
            .collect();
        ordered.sort_by(|a, b| natural_cmp(&a.path, &b.path));
        let count_ok = target.total_episodes.map_or(true, |t| t as usize == ordered.len());
        if count_ok && target.episode >= 1 {
            return ordered.get(target.episode as usize - 1).map(|f| f.index);
        }
    }
    None
}