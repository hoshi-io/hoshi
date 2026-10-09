use std::collections::HashSet;

use serde_json::Value;
use tracing::{debug, info, warn};
use crate::config::model::TorrentConfig;
use crate::config::service::ConfigService;
use crate::content::models::{EpisodeData, Metadata, RelationType, Relation};
use crate::content::repositories::content::ContentRepository;
use crate::error::CoreResult;
use crate::extensions::types::TorrentSearchResult;
use crate::extensions::ExtensionManager;
use crate::state::AppState;
use crate::torrent::parser::chain::{subtype_counts_as_season, ChainNode, PrequelLink, PrequelSource, season_position};
use crate::torrent::parser::matcher::{TargetDescriptor, MIN_LISTED, MIN_AUTO};
use crate::torrent::parser::parser::{is_continuation_title, ParsedRelease, season_from_title, parse_release_title, normalize_codec};
use crate::torrent::types::RankedTorrent;

const EARLY_STOP_CONFIDENCE: f32 = 0.85;

pub struct TorrentService;

fn total_episodes(m: &Metadata) -> Option<u32> {
    match &m.eps_or_chapters {
        EpisodeData::Count(n) => u32::try_from(*n).ok().filter(|n| *n > 0),
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

fn is_continuation(m: &Metadata) -> bool {
    collect_aliases(m).iter().any(|a| is_continuation_title(a))
}

fn prequel_link(relations: &[Relation]) -> PrequelLink {
    let mut prequels = relations
        .iter()
        .filter(|r| matches!(r.relation_type, RelationType::Prequel));
    // prefer one we've actually imported
    let all: Vec<&Relation> = prequels.by_ref().collect();
    if let Some(cid) = all.iter().find_map(|r| r.target_cid.clone()) {
        PrequelLink::Cid(cid)
    } else if !all.is_empty() {
        PrequelLink::Unresolved
    } else {
        PrequelLink::None
    }
}

/// first: romaji, english, main title, synonyms, native last.
fn collect_aliases(m: &Metadata) -> Vec<String> {
    let mut out = Vec::new();
    for key in ["romaji", "english"] {
        if let Some(t) = m.title_i18n.get(key) {
            out.push(t.clone());
        }
    }
    out.push(m.title.clone());
    out.extend(m.alt_titles.iter().cloned());
    if let Some(t) = m.title_i18n.get("native") {
        out.push(t.clone());
    }
    out.retain(|t| !t.trim().is_empty());
    out
}

struct DbPrequelSource<'a> {
    state: &'a AppState,
}

impl PrequelSource for DbPrequelSource<'_> {
    async fn node(&self, cid: &str) -> Option<ChainNode> {
        let fc =     ContentRepository::get_full_content(&self.state.pool, cid).await.ok().flatten()?;
        let m = fc.primary_metadata()?;
        Some(ChainNode {
            prequel: prequel_link(&fc.relations),
            episodes: total_episodes(m),
            counts_as_season: subtype_counts_as_season(m.subtype.as_deref()),
            is_continuation: is_continuation(m),
        })
    }
}

impl TorrentService {

    pub async fn build_target(
        state: &AppState,
        cid: &str,
        episode: u32,
    ) -> CoreResult<Option<TargetDescriptor>> {
        let Some(fc) = ContentRepository::get_full_content(&state.pool, cid).await? else {
            return Ok(None);
        };
        let Some(meta) = fc.primary_metadata() else {
            return Ok(None);
        };

        let aliases = collect_aliases(meta);
        let mut pos = season_position(
            prequel_link(&fc.relations),
            is_continuation(meta),
            &DbPrequelSource { state },
        )
            .await;
        
        if let Some(title_season) = aliases.iter().find_map(|a| season_from_title(a)) {
            if title_season != pos.season {
                warn!(
                    "torrent target: relation chain says season {} but title says {title_season}; using title",
                    pos.season
                );
                pos.season = title_season;
            }
        }
        let total = total_episodes(meta);
        info!(
            "torrent target: cid={cid} title={:?} subtype={:?} -> season={} episode={episode} absolute={:?} total={:?}",
            meta.title, meta.subtype, pos.season, pos.absolute_offset.map(|o| o + episode), total
        );
        if pos.absolute_offset.is_none() && pos.season > 1 {
            warn!("torrent target: prequel chain incomplete for cid={cid} (not imported or unknown episode count); absolute numbering disabled");
        }
        let allow_extras = !subtype_counts_as_season(meta.subtype.as_deref())
            || total == Some(1);

        let target = TargetDescriptor::new(
            aliases,
            pos.season,
            episode,
            pos.absolute_offset,
            total,
            allow_extras,
        );
        debug!("torrent target: aliases={:?} allow_extras={allow_extras}", target.aliases);
        Ok(Some(target))
    }

    /// Parse + match one batch of raw results. Extension-provided fields only
    /// fill gaps; our own parse of the title wins.
    pub fn rank(
        results: Vec<TorrentSearchResult>,
        target: &TargetDescriptor,
    ) -> Vec<RankedTorrent> {
        let mut out = Vec::new();
        for r in results {
            let mut parsed = parse_release_title(&r.title);
            Self::fill_from_extension(&mut parsed, &r);
            let m = target.evaluate(&parsed);
            if m.confidence < MIN_LISTED {
                debug!("  reject ({}) {:?}", m.reason, r.title);
            }
            if m.confidence >= MIN_LISTED {
                debug!(
                    "  keep {:.2} ({}) season={:?} ep={:?} range={:?} seeders={:?} {:?}",
                    m.confidence, m.reason, parsed.season, parsed.episode, parsed.episode_range,
                    r.seeders, r.title
                );
                out.push(RankedTorrent {
                    result: r,
                    parsed,
                    confidence: m.confidence,
                    reason: m.reason,
                });
            }
        }
        out
    }

    fn fill_from_extension(p: &mut ParsedRelease, r: &TorrentSearchResult) {
        if p.group.is_none() {
            p.group = r.release_group.clone();
        }
        if p.resolution.is_none() {
            p.resolution = r.resolution.clone();
        }
        if p.episode.is_none() && p.episode_range.is_none() && !p.is_batch {
            p.episode = r.episode_number;
        }
        if r.is_batch == Some(true) {
            p.is_batch = true;
        }
    }
    
    pub async fn search_ranked(
        manager: &ExtensionManager,
        ext_id: &str,
        target: &TargetDescriptor,
        filters: Value,
        page: u32,
        config: Option<&TorrentConfig>,
    ) -> CoreResult<Vec<RankedTorrent>> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut ranked: Vec<RankedTorrent> = Vec::new();
        let mut last_err = None;
        let mut any_ok = false;
        let mut raw_total = 0usize;

        for q in target.search_queries() {
            debug!("torrent search: query={q:?} ext={ext_id} page={page}");
            match manager.search_torrents(ext_id, &q, filters.clone(), page).await {
                Ok(results) => {
                    any_ok = true;
                    let raw = results.len();
                    let fresh: Vec<_> = results
                        .into_iter()
                        .filter(|r| {
                            let key = r.info_hash.clone().unwrap_or_else(|| r.id.clone());
                            seen.insert(key)
                        })
                        .collect();
                    raw_total += fresh.len();
                    let before = ranked.len();
                    ranked.extend(Self::rank(fresh.clone(), target));
                    info!(
                        "torrent search: query={q:?} -> {raw} raw, {} new, {} matched",
                        fresh.len(),
                        ranked.len() - before
                    );
                }
                Err(e) => {
                    warn!("torrent search: query={q:?} failed: {e:?}");
                    last_err = Some(e)
                }
            }

            // auto-select path: stop early once something is clearly right
            if let Some(cfg) = config {
                if ranked.iter().any(|t| {
                    t.confidence >= EARLY_STOP_CONFIDENCE && Self::passes_filters(t, cfg)
                }) {
                    info!("torrent search: confident match found, skipping remaining queries");
                    break;
                }
            }
        }

        if !any_ok {
            if let Some(e) = last_err {
                return Err(e);
            }
        }

        ranked.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.result.seeders.unwrap_or(0).cmp(&a.result.seeders.unwrap_or(0))
                })
        });
        Ok(ranked)
    }

    fn passes_filters(t: &RankedTorrent, config: &TorrentConfig) -> bool {
        let title = t.result.title.to_lowercase();
        if (t.result.seeders.unwrap_or(0).max(0) as u64) < config.min_seeders as u64 {
            return false;
        }
        if config
            .exclude_keywords
            .iter()
            .any(|kw| !kw.is_empty() && title.contains(&kw.to_lowercase()))
        {
            return false;
        }
        match config.require_batch {
            Some(want) => t.parsed.is_batch == want,
            None => true,
        }
    }

    /// Order: confidence bucket > group > resolution > codec/dual > version > seeders.
    pub fn pick_best<'a>(
        ranked: &'a [RankedTorrent],
        config: &TorrentConfig,
    ) -> Option<&'a RankedTorrent> {
        let want_res = config.preferred_resolution.trim().trim_end_matches(['p', 'P']);
        let want_codec = config.preferred_codec.as_deref().and_then(normalize_codec);

        let score = |t: &RankedTorrent| -> (i32, i32, i32, i32, u32, i32) {
            let conf = (t.confidence * 10.0).round() as i32;

            let group = t
                .parsed
                .group
                .as_deref()
                .and_then(|g| {
                    config.preferred_groups.iter().position(|p| p.eq_ignore_ascii_case(g))
                })
                .map(|pos| (config.preferred_groups.len() - pos) as i32)
                .unwrap_or(0);

            let res = t
                .parsed
                .resolution
                .as_deref()
                .map(|r| r.trim_end_matches(['p', 'P']).eq_ignore_ascii_case(want_res) as i32)
                .unwrap_or(0);

            let codec = match (&want_codec, &t.parsed.codec) {
                (Some(w), Some(c)) if w == c => 1,
                _ => 0,
            } + (config.prefer_dual_audio && t.parsed.dual_audio) as i32;

            let seeders = ((t.result.seeders.unwrap_or(0).max(0) as f64) + 1.0).log2() as i32;

            (conf, group, res, codec, t.parsed.version, seeders)
        };

        let eligible: Vec<&RankedTorrent> =
            ranked.iter().filter(|t| t.confidence >= MIN_AUTO).collect();
        let passing: Vec<&RankedTorrent> = eligible
            .iter()
            .copied()
            .filter(|t| Self::passes_filters(t, config))
            .collect();
        let best = passing.iter().copied().max_by_key(|t| score(t));

        match best {
            Some(t) => info!(
                "torrent pick: {:?} conf={:.2} ({}) score(conf,group,res,codec,ver,seeders)={:?} [{} ranked, {} auto-eligible, {} pass filters]",
                t.result.title, t.confidence, t.reason, score(t),
                ranked.len(), eligible.len(), passing.len()
            ),
            None => info!(
                "torrent pick: nothing selected [{} ranked, {} >= {MIN_AUTO} confidence, {} pass min_seeders/exclude/batch filters]",
                ranked.len(), eligible.len(), passing.len()
            ),
        }
        best
    }

    pub async fn auto_select_torrent(
        state: &AppState,
        manager: &ExtensionManager,
        user_id: i32,
        ext_id: &str,
        cid: &str,
        episode: u32,
        filters: Value,
        page: u32,
    ) -> CoreResult<Option<TorrentSearchResult>> {
        let config = ConfigService::get_config(state, user_id).await?.torrent;
        if !config.auto_select {
            debug!("torrent auto-select disabled in config");
            return Ok(None);
        }
        let Some(target) = Self::build_target(state, cid, episode).await? else {
            warn!("torrent auto-select: no content/metadata for cid={cid}");
            return Ok(None);
        };
        let ranked =
            Self::search_ranked(manager, ext_id, &target, filters, page, Some(&config)).await?;
        Ok(Self::pick_best(&ranked, &config).map(|t| t.result.clone()))
    }

    pub async fn sync_config(state: &AppState, user_id: i32) -> CoreResult<TorrentConfig> {
        let config = ConfigService::get_config(state, user_id).await?.torrent;
        state.torrent.update_config(config.clone()).await;
        Ok(config)
    }
}