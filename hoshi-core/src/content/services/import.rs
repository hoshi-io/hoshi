use sqlx::SqlitePool;
use serde_json::json;
use tracing::{info, instrument, warn};
use async_recursion::async_recursion;

use crate::content::models::{ContentType, EpisodeData, Metadata, Relation, RelationType, Status};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::relations::RelationRepository;
use crate::content::utils::generate_cid;
use crate::tracker::repository::TrackerRepository;
use crate::tracker::provider::TrackerMedia;
use crate::error::CoreResult;
use crate::tracker::types::TrackerMapping;

pub struct ImportService;

impl ImportService {
    #[instrument(skip(pool, media))]
    #[async_recursion]
    pub async fn import_media(
        pool: &SqlitePool,
        tracker_name: &str,
        media: &TrackerMedia,
    ) -> CoreResult<String> {
        let is_full = media.synopsis.is_some() || !media.characters.is_empty();

        let cid = if let Some(cid) = TrackerRepository::find_cid_by_tracker(pool, tracker_name, &media.tracker_id).await? {
            if is_full {
                let meta = Self::to_content_metadata(&cid, tracker_name, media);
                ContentRepository::upsert_metadata(pool, &meta).await?;
            }
            cid
        } else {
            let new_cid = generate_cid();
            info!(cid = %new_cid, title = %media.title, "Creating new entry (direct import)");

            let meta = Self::to_content_metadata(&new_cid, tracker_name, media);
            ContentRepository::create_with_type(pool, &media.content_type, media.nsfw, meta).await?;

            let now = chrono::Utc::now().timestamp();
            TrackerRepository::add_mapping(pool, &TrackerMapping {
                cid: new_cid.clone(),
                tracker_name: tracker_name.to_string(),
                tracker_id: media.tracker_id.clone(),
                tracker_url: None,
                created_at: now,
                updated_at: now,
            }).await?;

            let owner = TrackerRepository::find_cid_by_tracker(
                pool, tracker_name, &media.tracker_id
            ).await?;

            match owner {
                Some(ref existing_cid) if existing_cid != &new_cid => {
                    warn!(
                orphan = %new_cid,
                owner = %existing_cid,
                "Tracker ID already owned by another CID, discarding new entry"
            );
                    ContentRepository::delete(pool, &new_cid).await.ok();
                    existing_cid.clone()
                }
                _ => new_cid,
            }
        };

        for rel in &media.relations {
            let rel_type = match rel.relation_type.as_str() {
                "SEQUEL"      => RelationType::Sequel,
                "PREQUEL"     => RelationType::Prequel,
                "SIDE_STORY"  => RelationType::SideStory,
                "SPIN_OFF"    => RelationType::Spinoff,
                "ADAPTATION"  => RelationType::Adaptation,
                "PARENT"      => RelationType::Parent,
                "SUMMARY"     => RelationType::Summary,
                "SOURCE"      => RelationType::Source,
                "COMPILATION" => RelationType::Compilation,
                "CONTAINS"    => RelationType::Contains,
                "ALTERNATIVE" => RelationType::Alternative,
                "CHARACTER"   => RelationType::Character,
                "OTHER"       => RelationType::Other,
                other => {
                    warn!(relation_type = %other, "Unknown AniList relation type");
                    RelationType::Alternative
                }
            };

            let target_cid = TrackerRepository::find_cid_by_tracker(
                pool, tracker_name, &rel.media.tracker_id,
            ).await.unwrap_or(None);

            if let Err(e) = RelationRepository::upsert(pool, &Relation {
                id: None,
                source_cid: cid.to_string(),
                target_cid,
                target_tracker_name: tracker_name.to_string(),
                target_tracker_id: rel.media.tracker_id.clone(),
                relation_type: rel_type,
                target_title: rel.media.title.clone(),
                target_cover_image: rel.media.cover_image.clone(),
                source_name: tracker_name.to_string(),
                created_at: chrono::Utc::now().timestamp(),
            }).await {
                warn!(error = ?e, "Failed to save relation");
            }
        }

        Ok(cid)
    }

    pub fn to_content_metadata(cid: &str, tracker_name: &str, media: &TrackerMedia) -> Metadata {
        let now   = chrono::Utc::now().timestamp();
        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _                  => media.chapter_count.unwrap_or(0),
        };

        let status = media.status.as_deref().map(|s| match s {
            "FINISHED" | "ended" | "completed"   => Status::Completed,
            "RELEASING" | "ongoing" | "airing"   => Status::Ongoing,
            "NOT_YET_RELEASED" | "planned"       => Status::Planned,
            "CANCELLED" | "canceled"             => Status::Cancelled,
            "HIATUS"                             => Status::Hiatus,
            _                                    => Status::Ongoing,
        });

        Metadata {
            id: None, cid: cid.to_string(), source_name: tracker_name.to_string(),
            source_id: Some(media.tracker_id.clone()), subtype: media.format.clone(),
            title: media.title.clone(), alt_titles: media.alt_titles.clone(),
            title_i18n: media.title_i18n.clone(), synopsis: media.synopsis.clone(),
            cover_image: media.cover_image.clone(), banner_image: media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(count), status,
            genres: media.genres.clone(), release_date: media.release_date.clone(),
            end_date: media.end_date.clone(), rating: media.rating,
            trailer_url: media.trailer_url.clone(), characters: media.characters.clone(),
            studio: media.studio.clone(), staff: media.staff.clone(),
            external_ids: json!({}), episode_duration: Option::from(media.episode_duration.clone()), created_at: now, updated_at: now,
        }
    }
}