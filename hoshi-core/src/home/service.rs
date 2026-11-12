use std::collections::HashSet;
use std::sync::Arc;

use tracing::warn;

use crate::content::models::ContentType;
use crate::content::services::discover::DiscoverService;
use crate::content::types::{DiscoverItem, DiscoverQuery};
use crate::content::utils::show_adult;
use crate::error::{CoreError, CoreResult};
use crate::home::defaults::default_layout;
use crate::home::repository::HomeRepository;
use crate::home::types::{HomeLayout, Label, SequelCard};
use crate::state::AppState;

const MAX_TABS: usize = 8;
const MAX_SECTIONS: usize = 20;
const MAX_LABEL: usize = 64;

pub struct HomeLayoutService;

impl HomeLayoutService {
    pub fn default_layout() -> HomeLayout {
        default_layout()
    }

    pub async fn get_layout(state: &AppState, user_id: i32) -> CoreResult<HomeLayout> {
        let Some(raw) = HomeRepository::get_layout(&state.pool, user_id).await? else {
            return Ok(default_layout());
        };
        match serde_json::from_str(&raw) {
            Ok(layout) => Ok(layout),
            Err(e) => {
                warn!(user_id, error = ?e, "Stored home layout is unreadable, using default");
                Ok(default_layout())
            }
        }
    }

    pub async fn save_layout(state: &AppState, user_id: i32, layout: HomeLayout) -> CoreResult<HomeLayout> {
        validate(&layout)?;
        let raw = serde_json::to_string(&layout)
            .map_err(|_| CoreError::Internal("error.content.serialization".into()))?;
        HomeRepository::set_layout(&state.pool, user_id, &raw).await?;
        Ok(layout)
    }

    pub async fn reset_layout(state: &AppState, user_id: i32) -> CoreResult<HomeLayout> {
        HomeRepository::delete_layout(&state.pool, user_id).await?;
        Ok(default_layout())
    }

    pub async fn discover(
        state: &Arc<AppState>,
        user_id: i32,
        query: &DiscoverQuery,
        page: u32,
    ) -> CoreResult<Vec<DiscoverItem>> {
        DiscoverService::fetch(state, user_id, query, page, DiscoverService::ttl_for(query)).await
    }

    pub async fn unstarted_sequels(
        state: &AppState,
        user_id: i32,
        content_type: ContentType,
        limit: i64,
    ) -> CoreResult<Vec<SequelCard>> {
        let adult = show_adult(state, user_id).await;
        HomeRepository::unstarted_sequels(&state.pool, user_id, content_type.as_str(), adult, limit.clamp(1, 50)).await
    }
}

fn invalid() -> CoreError {
    CoreError::BadRequest("error.home.invalid_layout".into())
}

fn label_ok(label: &Label) -> bool {
    match label {
        Label::Key { key } => !key.is_empty() && key.len() <= MAX_LABEL,
        Label::Text { text } => !text.trim().is_empty() && text.chars().count() <= MAX_LABEL,
    }
}

fn validate(layout: &HomeLayout) -> CoreResult<()> {
    if layout.tabs.is_empty() || layout.tabs.len() > MAX_TABS {
        return Err(invalid());
    }

    let mut tab_ids = HashSet::new();
    for tab in &layout.tabs {
        if tab.id.is_empty() || !tab_ids.insert(tab.id.as_str()) || !label_ok(&tab.title) {
            return Err(invalid());
        }
        if tab.sections.len() > MAX_SECTIONS {
            return Err(invalid());
        }

        let mut section_ids = HashSet::new();
        for section in &tab.sections {
            if section.id.is_empty() || !section_ids.insert(section.id.as_str()) || !label_ok(&section.title) {
                return Err(invalid());
            }
        }

        if let Some(hero) = &tab.hero {
            if !section_ids.contains(hero.as_str()) {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
