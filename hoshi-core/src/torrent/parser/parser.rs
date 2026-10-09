use regex::Regex;
use std::sync::LazyLock;

macro_rules! re {
    ($p:expr) => {
        LazyLock::new(|| Regex::new($p).unwrap())
    };
}

static EXT: LazyLock<Regex> = re!(r"(?i)\.(mkv|mp4|avi|m4v|ts)$");
static GROUP: LazyLock<Regex> = re!(r"^\s*\[([^\]]+)\]");
static SCENE_GROUP: LazyLock<Regex> = re!(r"-([A-Za-z][A-Za-z0-9]+)$");
static EP_ONLY: LazyLock<Regex> = re!(r"(?i)^e\d+$");

static RES_P: LazyLock<Regex> = re!(r"(?i)\b(\d{3,4})p\b");
static RES_X: LazyLock<Regex> = re!(r"(?i)\b\d{3,4}x(\d{3,4})\b");
static CODEC: LazyLock<Regex> = re!(r"(?i)\b(x265|x264|h\.?265|h\.?264|hevc|avc|av1)\b");
static BIT10: LazyLock<Regex> = re!(r"(?i)\b10[- ]?bit\b");
static DUAL: LazyLock<Regex> = re!(r"(?i)\bdual[- ]?audio\b|\bdual\b");
static BATCH_KW: LazyLock<Regex> = re!(r"(?i)\b(batch|complete|collection|season pack|bd box)\b");
static EXTRA: LazyLock<Regex> = re!(
    r"(?i)\b(ncop\d*|nced\d*|creditless|menu|sample|trailer|pv\d*|cm\d*|ovas?|oads?|specials?|movie|extras?|bonus|preview|teaser)\b"
);

// episode patterns, tried in this order
static SE_RANGE: LazyLock<Regex> =
    re!(r"(?i)\bS(\d{1,2})\s?E(\d{1,4})\s?[-~]\s?E?(\d{1,4})\b");
static SE: LazyLock<Regex> = re!(r"(?i)\bS(\d{1,2})\s?E(\d{1,4})(?:v(\d{1,2}))?\b");
static RANGE: LazyLock<Regex> = re!(r"\b(\d{1,4})[-~](\d{1,4})\b");
static DASH_EP: LazyLock<Regex> =
    re!(r"(?i)\s[-–]\s(\d{1,4}(?:\.\d)?)(?:v(\d{1,2}))?(?:\s|\[|\(|$)");
static EP_WORD: LazyLock<Regex> =
    re!(r"(?i)(?:\b(?:episode|ep)\.?\s?|\bE)(\d{1,4})(?:v(\d{1,2}))?\b");
static HASH_EP: LazyLock<Regex> = re!(r"#(\d{1,4})(?:v(\d{1,2}))?\b");
static BRACKET_EP: LazyLock<Regex> = re!(r"\[(\d{1,4})(?:v(\d{1,2}))?\]");
static TRAIL_EP: LazyLock<Regex> = re!(r"\s(\d{1,3})(?:v(\d{1,2}))?\s*$");

static BRACKETS: LazyLock<Regex> = re!(r"\[[^\]]*\]|\([^)]*\)|\{[^}]*\}");

// season markers
static SEASON_ORD: LazyLock<Regex> = re!(r"(?i)\b(\d{1,2})(?:st|nd|rd|th)\s+season\b");
static SEASON_WORD: LazyLock<Regex> = re!(r"(?i)\bseason\s+(\d{1,2})\b");
static SEASON_S: LazyLock<Regex> = re!(r"(?i)\bS(\d{1,2})\b");
static SEASON_STRIP: LazyLock<Regex> = re!(
    r"(?i)\b\d{1,2}(?:st|nd|rd|th)\s+season\b|\bseason\s+\d{1,2}\b|\b(?:final|second|third|fourth)\s+season\b|\bS\d{1,2}\b|\bpart\s+\d{1,2}\b|\bcour\s+\d\b"
);

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedRelease {
    pub group: Option<String>,
    /// Title as written, season markers still included ("Re Zero 2nd Season").
    pub title_raw: String,
    /// Title with season/part markers removed ("Re Zero").
    pub title: String,
    pub season: Option<u32>,
    pub episode: Option<f64>,
    pub episode_range: Option<(f64, f64)>,
    pub version: u32,
    pub resolution: Option<String>,
    /// Normalized: "hevc" | "avc" | "av1"
    pub codec: Option<String>,
    pub dual_audio: bool,
    pub is_batch: bool,
    /// NCOP/NCED/PV/OVA/Movie/etc.
    pub is_extra: bool,
}

pub fn normalize_codec(raw: &str) -> Option<String> {
    let l = raw.to_lowercase().replace('.', "");
    match l.as_str() {
        "x265" | "h265" | "hevc" => Some("hevc".into()),
        "x264" | "h264" | "avc" => Some("avc".into()),
        "av1" => Some("av1".into()),
        _ => None,
    }
}

/// Lowercase, non-alphanumerics -> single spaces.
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = true;
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim().to_string()
}

pub fn strip_season_markers(s: &str) -> String {
    tidy(&SEASON_STRIP.replace_all(s, " "))
}

fn strip_brackets(s: &str) -> String {
    BRACKETS.replace_all(s, " ").into_owned()
}

fn tidy(s: &str) -> String {
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .trim_matches(|c: char| " -–—_.~:([{".contains(c))
        .to_string()
}

fn num(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

fn ver(c: Option<regex::Match>) -> Option<u32> {
    c.and_then(|m| m.as_str().parse().ok())
}

fn find_season(s: &str) -> Option<u32> {
    for re in [&*SEASON_ORD, &*SEASON_WORD, &*SEASON_S] {
        if let Some(c) = re.captures(s) {
            if let Ok(n) = c[1].parse::<u32>() {
                if n > 0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Explicit season number written in a title ("4th Season", "Season 4", "S4").
pub fn season_from_title(s: &str) -> Option<u32> {
    find_season(s)
}

static CONTINUATION: LazyLock<Regex> = re!(
    r"(?i)\bpart\s*([2-9])\b|\b([2-9])(?:nd|rd|th)\s+part\b|\bcour\s*([2-9])\b|\b([2-9])(?:nd|rd|th)\s+cour\b"
);

/// "... 2nd Season Part 2": the second half of a season that AniList lists as
/// a separate entry. It continues its prequel's season instead of starting one.
pub fn is_continuation_title(s: &str) -> bool {
    CONTINUATION.is_match(s)
}

pub fn parse_release_title(input: &str) -> ParsedRelease {
    let mut p = ParsedRelease {
        version: 1,
        ..Default::default()
    };

    let raw = EXT.replace(input.trim(), "").into_owned();
    let had_spaces = raw.contains(' ');
    let mut s = raw;

    // --- group ---
    let grp = GROUP
        .captures(&s)
        .map(|c| (c[1].trim().to_string(), c.get(0).unwrap().end()));
    if let Some((g, end)) = grp {
        p.group = Some(g);
        s = s[end..].to_string();
    } else if !had_spaces {
        let sg = SCENE_GROUP
            .captures(&s)
            .map(|c| (c[1].to_string(), c.get(0).unwrap().start()));
        if let Some((g, start)) = sg {
            if !EP_ONLY.is_match(&g) {
                p.group = Some(g);
                s.truncate(start);
            }
        }
    }

    // --- separators ---
    if had_spaces {
        s = s.replace('_', " ");
    } else {
        s = s.replace(['.', '_'], " ");
    }

    // --- flags & tech tags ---
    p.dual_audio = DUAL.is_match(&s);
    p.is_extra = EXTRA.is_match(&s);
    let batch_kw = BATCH_KW.is_match(&s);

    if let Some(c) = RES_P.captures(&s) {
        p.resolution = Some(format!("{}p", &c[1]));
    } else if let Some(c) = RES_X.captures(&s) {
        p.resolution = Some(format!("{}p", &c[1]));
    }
    if let Some(c) = CODEC.captures(&s) {
        p.codec = normalize_codec(&c[1]);
    }
    for re in [&*RES_P, &*RES_X, &*CODEC, &*BIT10] {
        s = re.replace_all(&s, " ").into_owned();
    }

    // --- episode ---
    let mut ep_start: Option<usize> = None;
    let mut title_src: Option<String> = None; // used when positions refer to a different string

    if let Some(c) = SE_RANGE.captures(&s) {
        p.season = c[1].parse().ok();
        p.episode_range = Some((num(&c[2]), num(&c[3])));
        ep_start = Some(c.get(0).unwrap().start());
    } else if let Some(c) = SE.captures(&s) {
        p.season = c[1].parse().ok();
        p.episode = Some(num(&c[2]));
        p.version = ver(c.get(3)).unwrap_or(1);
        ep_start = Some(c.get(0).unwrap().start());
    } else if let Some(c) = RANGE.captures_iter(&s).find(|c| {
        let (a, b) = (num(&c[1]), num(&c[2]));
        a < 1900.0 && b > a
    }) {
        p.episode_range = Some((num(&c[1]), num(&c[2])));
        ep_start = Some(c.get(0).unwrap().start());
    } else if let Some(c) = DASH_EP.captures(&s) {
        p.episode = Some(num(&c[1]));
        p.version = ver(c.get(2)).unwrap_or(1);
        ep_start = Some(c.get(0).unwrap().start());
    } else if let Some(c) = EP_WORD.captures(&s) {
        p.episode = Some(num(&c[1]));
        p.version = ver(c.get(2)).unwrap_or(1);
        ep_start = Some(c.get(0).unwrap().start());
    } else if let Some(c) = HASH_EP.captures(&s) {
        p.episode = Some(num(&c[1]));
        p.version = ver(c.get(2)).unwrap_or(1);
        ep_start = Some(c.get(0).unwrap().start());
    } else if let Some(c) = BRACKET_EP.captures(&s) {
        p.episode = Some(num(&c[1]));
        p.version = ver(c.get(2)).unwrap_or(1);
        ep_start = Some(c.get(0).unwrap().start());
    } else {
        let clean = strip_brackets(&s);
        if let Some(c) = TRAIL_EP.captures(&clean) {
            p.episode = Some(num(&c[1]));
            p.version = ver(c.get(2)).unwrap_or(1);
            ep_start = Some(c.get(0).unwrap().start());
            title_src = Some(clean);
        }
    }

    if p.season.is_none() {
        p.season = find_season(&s);
    }
    p.is_batch = batch_kw || p.episode_range.is_some();

    let src = title_src.as_deref().unwrap_or(&s);
    let head = match ep_start {
        Some(i) => &src[..i],
        None => src,
    };
    p.title_raw = tidy(&strip_brackets(head));
    p.title = strip_season_markers(&p.title_raw);
    p
}