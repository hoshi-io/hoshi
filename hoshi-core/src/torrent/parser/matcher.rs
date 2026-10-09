use std::collections::HashSet;
use crate::torrent::parser::parser::{normalize, strip_season_markers, ParsedRelease};

pub const MIN_LISTED: f32 = 0.4;
pub const MIN_AUTO: f32 = 0.6;
const TITLE_THRESHOLD: f32 = 0.75;
const ENTRY_THRESHOLD: f32 = 0.95;

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResult {
    pub confidence: f32,
    pub reason: &'static str,
}

fn reject(reason: &'static str) -> MatchResult {
    MatchResult { confidence: 0.0, reason }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSummary {
    pub aliases: Vec<String>,
    pub season: u32,
    pub episode: u32,
    pub absolute_episode: Option<u32>,
    pub total_episodes: Option<u32>,
    pub queries: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TargetDescriptor {
    /// Raw titles for this entry, priority order (romaji, english, synonyms...).
    pub aliases: Vec<String>,
    pub season: u32,
    /// Episode as the tracker (AniList) numbers it: restarts at 1 each season.
    pub episode: u32,
    /// Episode in continuous numbering, when the chain is fully known.
    pub absolute_episode: Option<u32>,
    pub total_episodes: Option<u32>,
    /// Movies / OVAs / specials: don't reject "extra"-looking releases.
    pub allow_extras: bool,
    series_keys: Vec<String>, // normalized, season markers stripped
    entry_keys: Vec<String>,  // normalized, markers kept
}

impl TargetDescriptor {
    pub fn new(
        aliases: Vec<String>,
        season: u32,
        episode: u32,
        absolute_offset: Option<u32>,
        total_episodes: Option<u32>,
        allow_extras: bool,
    ) -> Self {
        let mut seen = HashSet::new();
        let aliases: Vec<String> = aliases
            .into_iter()
            .filter(|a| {
                let k = normalize(a);
                !k.is_empty() && seen.insert(k)
            })
            .collect();
        let series_keys = aliases
            .iter()
            .map(|a| normalize(&strip_season_markers(a)))
            .filter(|k| !k.is_empty())
            .collect();
        let entry_keys = aliases.iter().map(|a| normalize(a)).collect();
        Self {
            aliases,
            season: season.max(1),
            episode,
            absolute_episode: absolute_offset.map(|o| o + episode),
            total_episodes,
            allow_extras,
            series_keys,
            entry_keys,
        }
    }

    pub fn describe(&self) -> TargetSummary {
        TargetSummary {
            aliases: self.aliases.clone(),
            season: self.season,
            episode: self.episode,
            absolute_episode: self.absolute_episode,
            total_episodes: self.total_episodes,
            queries: self.search_queries(),
        }
    }

    pub fn evaluate(&self, p: &ParsedRelease) -> MatchResult {
        if p.is_extra && !self.allow_extras {
            return reject("extra content");
        }

        let sim = best_similarity(&normalize(&p.title), &self.series_keys);
        if sim < TITLE_THRESHOLD {
            return reject("title mismatch");
        }
        // The release uses a title unique to this entry
        let entry_specific =
            best_similarity(&normalize(&p.title_raw), &self.entry_keys) >= ENTRY_THRESHOLD;

        let ep = self.episode as f64;
        let abs = self.absolute_episode.map(|a| a as f64);
        let first_season = self.season == 1;
        let unmarked_ok = first_season || entry_specific;

        let (base, reason): (f32, &'static str) = if let Some((a, b)) = p.episode_range {
            let season_ok = match p.season {
                Some(s) => s == self.season,
                None => unmarked_ok,
            };
            if season_ok && a <= ep && ep <= b {
                (0.65, "batch contains episode")
            } else if p.season.is_none() && abs.is_some_and(|x| a <= x && x <= b) {
                (0.6, "batch contains absolute episode")
            } else {
                return reject("batch range misses episode");
            }
        } else if let Some(pe) = p.episode {
            // ---- single episode ----
            match p.season {
                Some(s) if s != self.season => return reject("season mismatch"),
                Some(_) => {
                    if pe == ep {
                        (1.0, "season + episode")
                    } else if abs == Some(pe) {
                        (0.8, "season + absolute episode")
                    } else {
                        return reject("episode mismatch");
                    }
                }
                None => {
                    if unmarked_ok && pe == ep {
                        (0.9, "episode, no season marker")
                    } else if abs == Some(pe) {
                        (0.85, "absolute episode")
                    } else if pe == ep {
                        (0.45, "ambiguous: may belong to an earlier season")
                    } else {
                        return reject("episode mismatch");
                    }
                }
            }
        } else if p.is_batch {
            match p.season {
                Some(s) if s == self.season => (0.6, "season batch"),
                Some(_) => return reject("season mismatch"),
                None if unmarked_ok => (0.55, "unnumbered batch"),
                None => return reject("batch of unknown season"),
            }
        } else if self.total_episodes == Some(1) {
            // ---- movie-like entry ----
            (0.75, "single-episode entry, title match")
        } else {
            return reject("no episode number");
        };

        MatchResult { confidence: base * (0.7 + 0.3 * sim), reason }
    }
    
    pub fn search_queries(&self) -> Vec<String> {
        let mut titles: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for a in &self.aliases {
            let t = strip_season_markers(a);
            let safe: String = t
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { ' ' })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if !safe.is_empty() && seen.insert(normalize(&safe)) {
                titles.push(safe);
            }
            if titles.len() == 2 {
                break;
            }
        }

        let width = |n: u32| (n.to_string().len()).max(2);
        let w = width(self.total_episodes.unwrap_or(0).max(self.episode));
        let ep = format!("{:0w$}", self.episode, w = w);

        let mut out: Vec<String> = Vec::new();
        if self.season > 1 {
            // anime indexers mostly use absolute numbers, so try those first
            if let Some(a) = self.absolute_episode {
                let aw = width(a);
                for t in &titles {
                    out.push(format!("{t} {:0aw$}", a, aw = aw));
                }
            }
            for t in &titles {
                out.push(format!("{t} S{:02}E{ep}", self.season));
            }
        }
        for t in &titles {
            out.push(format!("{t} {ep}"));
        }
        let mut seen = HashSet::new();
        out.retain(|q| seen.insert(q.clone()));
        out.truncate(6);
        out
    }
}

fn best_similarity(a: &str, keys: &[String]) -> f32 {
    keys.iter().map(|k| similarity(a, k)).fold(0.0, f32::max)
}

fn similarity(a: &str, b: &str) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a.replace(' ', "") == b.replace(' ', "") {
        return 1.0;
    }
    let ta: HashSet<&str> = a.split(' ').collect();
    let tb: HashSet<&str> = b.split(' ').collect();
    let inter = ta.intersection(&tb).count() as f32;
    2.0 * inter / (ta.len() + tb.len()) as f32
}