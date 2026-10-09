use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum PrequelLink {
    None,
    Unresolved,
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
        match link {
            PrequelLink::None => break,
            PrequelLink::Unresolved => {
                season += 1;
                offset = None;
                break;
            }
            PrequelLink::Cid(cid) => {
                if !seen.insert(cid.clone()) || seen.len() > 30 {
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
        }
    }
    SeasonPosition { season, absolute_offset: offset }
}