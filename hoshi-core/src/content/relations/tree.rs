use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tracing::instrument;

use crate::content::models::RelationType;
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::relations::RelationRepository;
use crate::content::types::{RelationEdge, RelationGraph, RelationNode};
use crate::error::CoreResult;
use crate::state::AppState;

use super::resolve::{resolve_target, ResolveBudget};

/// Cap on *content* nodes in the graph (leaf placeholders don't count).
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
        let budget = ResolveBudget::new(MAX_TREE_EAGER_RESOLVES);

        // `visited` = popped from the queue (dedup); `node_cids` = actually emitted as a node.
        // The cap is on node_cids, so failed fetches don't eat into it.
        let mut visited: HashSet<String> = HashSet::new();
        let mut node_cids: HashSet<String> = HashSet::new();
        let mut visited_leaves: HashSet<(String, String)> = HashSet::new();
        let mut seen_edges: HashSet<(String, String)> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        queue.push_back((root_cid.to_string(), 0));

        while let Some((cid, depth)) = queue.pop_front() {
            if node_cids.len() >= MAX_TREE_NODES {
                break; // BFS: nothing further can be added
            }
            if !visited.insert(cid.clone()) {
                continue;
            }

            let Some(full) = ContentRepository::get_full_content(&state.pool, &cid).await? else {
                continue;
            };

            node_cids.insert(cid.clone());
            nodes.push(RelationNode {
                cid: Some(cid.clone()),
                tracker_name: None,
                tracker_id: None,
                title: full.metadata.first().map(|m| m.title.clone()).unwrap_or_default(),
                cover_image: full.metadata.first().and_then(|m| m.cover_image.clone()),
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
                if target_cid.is_none() && budget.try_take() {
                    target_cid =
                        resolve_target(state, &rel.target_tracker_name, &rel.target_tracker_id).await;
                }

                // Unordered pair: collapses mirrored edges (A sequel B / B prequel A).
                let edge_key = match &target_cid {
                    Some(t) if cid < *t => (cid.clone(), t.clone()),
                    Some(t) => (t.clone(), cid.clone()),
                    None => (
                        cid.clone(),
                        format!("{}:{}", rel.target_tracker_name, rel.target_tracker_id),
                    ),
                };
                if !seen_edges.insert(edge_key) {
                    continue;
                }

                edges.push(RelationEdge {
                    source_cid: cid.clone(),
                    target_cid: target_cid.clone(),
                    target_tracker_name: rel.target_tracker_name.clone(),
                    target_tracker_id: rel.target_tracker_id.clone(),
                    relation_type: rel.relation_type.clone(),
                });

                match &target_cid {
                    Some(t) if !visited.contains(t) => queue.push_back((t.clone(), depth + 1)),
                    None => {
                        let leaf_key =
                            (rel.target_tracker_name.clone(), rel.target_tracker_id.clone());
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

        // Edges pushed toward targets that never became nodes (node cap hit,
        // or get_full_content returned None) would dangle in the UI: drop them.
        edges.retain(|e| e.target_cid.as_ref().map_or(true, |t| node_cids.contains(t)));

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
}