use std::collections::HashSet;

use crate::content::models::{Relation, RelationType};

/// Max prequel hops resolved over the network per `build_target` call.
pub const CHAIN_RESOLVE_BUDGET: usize = 8;
/// Hard stop for the walk (cycles / absurd chains).
const MAX_CHAIN_DEPTH: usize = 30;

/// A prequel we know about from the tracker but haven't imported yet.
#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedRef {
    pub tracker_name: String,
    pub tracker_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrequelLink {
    None,
    Unresolved(UnresolvedRef),
    Cid(String),
}

#[derive(Debug, Clone)]
pub struct ChainNode {
    pub prequel: PrequelLink,
    pub episodes: Option<u32>,
    /// false for movies / OVAs / specials: they don't count as a season.
    pub counts_as_season: bool,
    /// "Part 2" / "cour 2" entry: continues its prequel's season.
    pub is_continuation: bool,
}

#[allow(async_fn_in_trait)]
pub trait PrequelSource {
    async fn node(&self, cid: &str) -> Option<ChainNode>;
    /// Import an unresolved prequel and return its cid. May hit the network;
    /// implementors should enforce a budget and return None when exhausted.
    async fn resolve(&self, r: &UnresolvedRef) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonPosition {
    pub season: u32,
    pub absolute_offset: Option<u32>,
}

pub fn subtype_counts_as_season(subtype: Option<&str>) -> bool {
    match subtype.map(|s| s.to_lowercase()) {
        None => true,
        Some(s) => matches!(s.as_str(), "tv" | "tv_short" | "tv short" | "ona" | "web"),
    }
}

/// Pick the prequel to follow. Deterministic: imported ones first, then lowest relation id.
pub fn prequel_link(relations: &[Relation]) -> PrequelLink {
    let mut prequels: Vec<&Relation> = relations
        .iter()
        .filter(|r| matches!(r.relation_type, RelationType::Prequel))
        .collect();
    prequels.sort_by_key(|r| (r.target_cid.is_none(), r.id));

    match prequels.first() {
        None => PrequelLink::None,
        Some(r) => match &r.target_cid {
            Some(cid) => PrequelLink::Cid(cid.clone()),
            None => PrequelLink::Unresolved(UnresolvedRef {
                tracker_name: r.target_tracker_name.clone(),
                tracker_id: r.target_tracker_id.clone(),
            }),
        },
    }
}

pub async fn season_position(
    start: PrequelLink,
    start_is_continuation: bool,
    src: &impl PrequelSource,
) -> SeasonPosition {
    let mut season = if start_is_continuation { 0 } else { 1 };
    let mut offset = Some(0u32);
    let mut link = start;
    let mut seen: HashSet<String> = HashSet::new();

    loop {
        let cid = match link {
            PrequelLink::None => break,
            PrequelLink::Cid(cid) => cid,
            PrequelLink::Unresolved(r) => match src.resolve(&r).await {
                Some(cid) => cid,
                None => {
                    // unreachable prequel: assume one season behind us, numbering unknown
                    season += 1;
                    offset = None;
                    break;
                }
            },
        };

        if !seen.insert(cid.clone()) || seen.len() > MAX_CHAIN_DEPTH {
            // cycle / too deep: chain is incomplete, so the offset can't be trusted
            offset = None;
            break;
        }

        match src.node(&cid).await {
            None => {
                season += 1;
                offset = None;
                break;
            }
            Some(n) => {
                if n.counts_as_season {
                    if !n.is_continuation {
                        season += 1;
                    }
                    offset = match (offset, n.episodes) {
                        (Some(o), Some(e)) => Some(o + e),
                        _ => None,
                    };
                }
                link = n.prequel;
            }
        }
    }

    SeasonPosition { season, absolute_offset: offset }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Fake {
        nodes: HashMap<String, ChainNode>,
        resolvable: HashMap<String, String>, // tracker_id -> cid
    }

    impl PrequelSource for Fake {
        async fn node(&self, cid: &str) -> Option<ChainNode> {
            self.nodes.get(cid).cloned()
        }
        async fn resolve(&self, r: &UnresolvedRef) -> Option<String> {
            self.resolvable.get(&r.tracker_id).cloned()
        }
    }

    fn tv(prequel: PrequelLink, eps: u32) -> ChainNode {
        ChainNode { prequel, episodes: Some(eps), counts_as_season: true, is_continuation: false }
    }

    fn missing(id: &str) -> PrequelLink {
        PrequelLink::Unresolved(UnresolvedRef { tracker_name: "t".into(), tracker_id: id.into() })
    }

    #[tokio::test]
    async fn resolves_missing_prequel_and_keeps_walking() {
        let src = Fake {
            nodes: HashMap::from([
                ("s2".into(), tv(PrequelLink::Cid("s1".into()), 12)),
                ("s1".into(), tv(PrequelLink::None, 12)),
            ]),
            resolvable: HashMap::from([("s2-id".into(), "s2".into())]),
        };
        let pos = season_position(missing("s2-id"), false, &src).await;
        assert_eq!(pos, SeasonPosition { season: 3, absolute_offset: Some(24) });
    }

    #[tokio::test]
    async fn unreachable_prequel_disables_offset() {
        let src = Fake { nodes: HashMap::new(), resolvable: HashMap::new() };
        let pos = season_position(missing("nope"), false, &src).await;
        assert_eq!(pos, SeasonPosition { season: 2, absolute_offset: None });
    }

    #[tokio::test]
    async fn cycle_disables_offset() {
        let src = Fake {
            nodes: HashMap::from([("a".into(), tv(PrequelLink::Cid("a".into()), 12))]),
            resolvable: HashMap::new(),
        };
        let pos = season_position(PrequelLink::Cid("a".into()), false, &src).await;
        assert_eq!(pos.absolute_offset, None);
    }
}