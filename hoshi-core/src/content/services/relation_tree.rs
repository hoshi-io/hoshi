use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tracing::instrument;

use crate::content::models::{Relation, RelationType};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::relations::RelationRepository;
use crate::content::services::enrichment::EnrichmentService;
use crate::content::services::resolver::ContentResolverService;
use crate::content::types::{RelationEdge, RelationGraph, RelationNode};
use crate::error::CoreResult;
use crate::state::AppState;

const MAX_TREE_NODES: usize = 150;
const MAX_TREE_DEPTH: usize = 4;
const MAX_TREE_EAGER_RESOLVES: usize = 12;

pub struct RelationTreeService;

impl RelationTreeService {
    #[instrument(skip(state))]
    pub async fn get_relation_tree(
        state: &Arc<AppState>,
        root_cid: &str,
    ) -> CoreResult<RelationGraph> {
        let mut visited_cids: HashSet<String> = HashSet::new();
        let mut visited_leaves: HashSet<(String, String)> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut seen_edges: HashSet<(String, String)> = HashSet::new();
        let mut eager_resolves = 0usize;

        queue.push_back((root_cid.to_string(), 0));

        while let Some((cid, depth)) = queue.pop_front() {
            if visited_cids.contains(&cid) || visited_cids.len() >= MAX_TREE_NODES {
                continue;
            }
            visited_cids.insert(cid.clone());

            let Some(full) = ContentRepository::get_full_content(&state.pool, &cid).await? else {
                continue;
            };

            let title = full.metadata.first().map(|m| m.title.clone()).unwrap_or_default();
            let cover = full.metadata.first().and_then(|m| m.cover_image.clone());

            nodes.push(RelationNode {
                cid: Some(cid.clone()),
                tracker_name: None,
                tracker_id: None,
                title,
                cover_image: cover,
            });

            if depth >= MAX_TREE_DEPTH {
                continue;
            }

            let relations = RelationRepository::get_by_source(&state.pool, &cid).await?;

            for rel in relations {
                if !Self::is_traversable(&rel.relation_type) {
                    continue;
                }

                let mut target_cid = rel.target_cid.clone();
                if target_cid.is_none() && eager_resolves < MAX_TREE_EAGER_RESOLVES {
                    eager_resolves += 1;
                    target_cid = Self::eager_resolve_relation_target(state, &rel).await;
                }

                let edge_key = match &target_cid {
                    Some(tcid) if cid < *tcid => (cid.clone(), tcid.clone()),
                    Some(tcid) => (tcid.clone(), cid.clone()),
                    None => (cid.clone(), format!("{}:{}", rel.target_tracker_name, rel.target_tracker_id)),
                };

                if seen_edges.contains(&edge_key) {
                    continue;
                }
                seen_edges.insert(edge_key);

                edges.push(RelationEdge {
                    source_cid: cid.clone(),
                    target_cid: target_cid.clone(),
                    target_tracker_name: rel.target_tracker_name.clone(),
                    target_tracker_id: rel.target_tracker_id.clone(),
                    relation_type: rel.relation_type.clone(),
                });

                match &target_cid {
                    Some(tcid) if !visited_cids.contains(tcid) => {
                        queue.push_back((tcid.clone(), depth + 1));
                    }
                    None => {
                        let leaf_key = (rel.target_tracker_name.clone(), rel.target_tracker_id.clone());
                        if visited_leaves.insert(leaf_key) {
                            nodes.push(RelationNode {
                                cid: None,
                                tracker_name: Some(rel.target_tracker_name.clone()),
                                tracker_id: Some(rel.target_tracker_id.clone()),
                                title: rel.target_title.clone(),
                                cover_image: rel.target_cover_image.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(RelationGraph { nodes, edges })
    }

    fn is_traversable(rel_type: &RelationType) -> bool {
        matches!(
            rel_type,
            RelationType::Prequel
                | RelationType::Sequel
                | RelationType::Parent
                | RelationType::SideStory
                | RelationType::Summary
                | RelationType::Alternative
                | RelationType::Adaptation
                | RelationType::Source
                | RelationType::Compilation
                | RelationType::Contains
        )
        // excluded on purpose: Character, SpinOff, Other
    }

    async fn eager_resolve_relation_target(state: &Arc<AppState>, rel: &Relation) -> Option<String> {
        let media = ContentResolverService::fetch_tracker_media(
            state, &rel.target_tracker_name, &rel.target_tracker_id,
        ).await.ok()?;

        let full = EnrichmentService::create_enriched_content(
            state, &media.content_type, &media,
            &rel.target_tracker_id, &rel.target_tracker_name, None,
        ).await.ok()?;

        let _ = RelationRepository::backfill_target_cid(
            &state.pool, &rel.target_tracker_name, &rel.target_tracker_id, &full.content.cid,
        ).await;

        Some(full.content.cid)
    }
}