use crate::content::models::{Character, ContentType, EpisodeData, Metadata, StaffMember, Status};
use crate::error::{CoreError, CoreResult};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::{
    TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, TrackerRelation, UpdateEntryParams,
    UserListEntry,
};

const CLIENT_ID: &str = "dd031b32d2f56c990b1425efe6c42ad847e7fe3ab46bf1299f05ecd856bdb7dd";
const BASE_URL: &str = "https://kitsu.io/api/edge";
const OAUTH_URL: &str = "https://kitsu.io/api/oauth/token";

const ACCEPT: &str = "application/vnd.api+json";
const CONTENT_TYPE: &str = "application/vnd.api+json";

pub struct KitsuProvider {
    client: Client,
}

impl KitsuProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn get_public(&self, path: &str) -> CoreResult<Value> {
        let res = self
            .client
            .get(format!("{}{}", BASE_URL, path))
            .header("Accept", ACCEPT)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(CoreError::NotFound("Kitsu: resource not found".into()));
        }
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Kitsu HTTP {}: {}", status, text)));
        }

        res.json::<Value>()
            .await
            .map_err(|e| CoreError::Internal(format!("Kitsu JSON parse: {}", e)))
    }

    async fn get_auth(&self, path: &str, token: &str) -> CoreResult<Value> {
        let res = self
            .client
            .get(format!("{}{}", BASE_URL, path))
            .header("Accept", ACCEPT)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Kitsu HTTP {}: {}", status, text)));
        }

        res.json::<Value>()
            .await
            .map_err(|e| CoreError::Internal(format!("Kitsu JSON parse: {}", e)))
    }

    async fn mutate(
        &self,
        method: reqwest::Method,
        path: &str,
        token: &str,
        body: Option<&Value>,
    ) -> CoreResult<Value> {
        let mut req = self
            .client
            .request(method, format!("{}{}", BASE_URL, path))
            .header("Accept", ACCEPT)
            .header("Content-Type", CONTENT_TYPE)
            .header("Authorization", format!("Bearer {}", token));

        if let Some(b) = body {
            req = req.json(b);
        }

        let res = req
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(CoreError::Internal(format!("Kitsu HTTP {}: {}", status, text)));
        }

        if res.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(json!({}));
        }

        res.json::<Value>()
            .await
            .map_err(|e| CoreError::Internal(format!("Kitsu JSON parse: {}", e)))
    }

    fn find_included<'a>(
        included: &'a [Value],
        type_name: &str,
        rel_ids: &[&str],
    ) -> Vec<&'a Value> {
        included.iter().filter(|item| {
            let t  = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
            t == type_name && rel_ids.contains(&id)
        }).collect()
    }

    fn find_one_included<'a>(included: &'a [Value], type_name: &str, id: &str) -> Option<&'a Value> {
        included.iter().find(|item| {
            item.get("type").and_then(|v| v.as_str()) == Some(type_name)
                && item.get("id").and_then(|v| v.as_str()) == Some(id)
        })
    }

    fn rel_ids<'a>(data: &'a Value, rel_key: &str) -> Vec<&'a str> {
        data.get("relationships")
            .and_then(|r| r.get(rel_key))
            .and_then(|c| c.get("data"))
            .and_then(|d| d.as_array())
            .map(|arr| arr.iter().filter_map(|i| i.get("id")?.as_str()).collect())
            .unwrap_or_default()
    }

    fn media_from_data(&self, data: &Value, included: Option<&Vec<Value>>) -> Option<TrackerMedia> {
        let id         = data.get("id")?.as_str()?.to_string();
        let attrs      = data.get("attributes")?;
        let media_type = data.get("type")?.as_str()?;

        let content_type = match media_type {
            "anime" => ContentType::Anime,
            "manga" => {
                let subtype = attrs.get("subtype").and_then(|v| v.as_str()).unwrap_or("");
                if matches!(subtype, "novel" | "light_novel") { ContentType::Novel } else { ContentType::Manga }
            }
            _ => return None,
        };

        let titles    = attrs.get("titles");
        let canonical = attrs.get("canonicalTitle").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();

        let mut title_i18n: HashMap<String, String> = HashMap::new();
        if let Some(t) = titles {
            if let Some(s) = t.get("ja_jp").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                title_i18n.insert("native".to_string(), s.to_string());
            }
            if let Some(s) = t.get("en_jp").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                title_i18n.insert("romaji".to_string(), s.to_string());
            }
            if let Some(s) = t.get("en").or(t.get("en_us")).and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                title_i18n.insert("english".to_string(), s.to_string());
            }
        }

        let mut alt_titles: Vec<String> = Vec::new();
        if let Some(t) = titles {
            for key in &["en", "en_jp", "en_us", "ja_jp"] {
                if let Some(s) = t.get(key).and_then(|v| v.as_str()) {
                    if s != canonical && !alt_titles.contains(&s.to_string()) {
                        alt_titles.push(s.to_string());
                    }
                }
            }
        }
        if let Some(abbrevs) = attrs.get("abbreviatedTitles").and_then(|v| v.as_array()) {
            for a in abbrevs {
                if let Some(s) = a.as_str() {
                    if !alt_titles.contains(&s.to_string()) {
                        alt_titles.push(s.to_string());
                    }
                }
            }
        }

        let cover_image = attrs
            .get("posterImage")
            .and_then(|p| p.get("large").or(p.get("original")).or(p.get("medium")))
            .and_then(|v| v.as_str())
            .map(String::from);

        let banner_image = attrs
            .get("coverImage")
            .and_then(|c| c.get("large").or(c.get("original")))
            .and_then(|v| v.as_str())
            .map(String::from);

        let rating = attrs
            .get("averageRating")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|v| (v / 10.0) as f32);

        let trailer_url = attrs
            .get("youtubeVideoId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|id| format!("https://www.youtube.com/watch?v={}", id));

        let nsfw = attrs.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false)
            || attrs.get("ageRating").and_then(|v| v.as_str()).map(|r| r == "R18").unwrap_or(false);

        let episode_duration = attrs.get("episodeLength").and_then(|v| v.as_i64()).map(|i| i as i32);

        let genres     = self.extract_genres(data, included);
        let relations  = self.extract_relations(data, included);
        let cross_ids  = self.extract_cross_ids(data, included);
        let characters = self.extract_characters(data, included);
        let staff      = self.extract_staff(data, included);
        let studio     = self.extract_studio(data, included);

        let tracker_url = Some(format!(
            "https://kitsu.io/{}/{}",
            media_type,
            attrs.get("slug").and_then(|v| v.as_str()).unwrap_or(&id)
        ));

        Some(TrackerMedia {
            tracker_id: id,
            tracker_url,
            cross_ids,
            content_type,
            title: canonical,
            alt_titles,
            title_i18n,
            synopsis:         attrs.get("synopsis").or(attrs.get("description")).and_then(|v| v.as_str()).map(String::from),
            cover_image,
            banner_image,
            episode_count:    attrs.get("episodeCount").and_then(|v| v.as_i64()).map(|i| i as i32),
            chapter_count:    attrs.get("chapterCount").and_then(|v| v.as_i64()).map(|i| i as i32),
            status:           attrs.get("status").and_then(|v| v.as_str()).map(String::from),
            genres,
            tags:             vec![],
            nsfw,
            release_date:     attrs.get("startDate").and_then(|v| v.as_str()).map(String::from),
            end_date:         attrs.get("endDate").and_then(|v| v.as_str()).map(String::from),
            rating,
            trailer_url,
            format:           attrs.get("subtype").and_then(|v| v.as_str()).map(String::from),
            studio,
            characters,
            staff,
            relations,
            episode_duration,
        })
    }

    fn extract_genres(&self, data: &Value, included: Option<&Vec<Value>>) -> Vec<String> {
        let included = match included { Some(i) => i, None => return vec![] };

        let ids = Self::rel_ids(data, "categories")
            .into_iter()
            .chain(Self::rel_ids(data, "genres"))
            .collect::<Vec<_>>();

        included.iter()
            .filter(|item| {
                let t  = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                (t == "categories" || t == "genres") && ids.contains(&id)
            })
            .filter_map(|item| {
                item.get("attributes")
                    .and_then(|a| a.get("title").or(a.get("name")))
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect()
    }

    fn extract_linked_people<'a>(
        data: &Value,
        included: &'a [Value],
        rel_key: &str,
        join_type: &str,
        entity_type: &str,
        entity_rel_key: &str,
    ) -> Vec<(String, &'a Value)> { // (role, entity node)
        let join_ids = Self::rel_ids(data, rel_key);

        join_ids.into_iter().take(10).filter_map(|join_id| {
            let join_node = Self::find_one_included(included, join_type, join_id)?;
            let role = join_node.get("attributes")
                .and_then(|a| a.get("role"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let entity_id = join_node
                .get("relationships").and_then(|r| r.get(entity_rel_key))
                .and_then(|c| c.get("data"))
                .and_then(|d| d.get("id"))
                .and_then(|v| v.as_str())?;
            let entity = Self::find_one_included(included, entity_type, entity_id)?;
            Some((role, entity))
        }).collect()
    }

    fn extract_characters(&self, data: &Value, included: Option<&Vec<Value>>) -> Vec<Character> {
        let included = match included { Some(i) => i, None => return vec![] };

        Self::extract_linked_people(data, included, "characters", "mediaCharacters", "characters", "character")
            .into_iter()
            .map(|(role, node)| {
                let attrs = node.get("attributes");
                let name  = attrs.and_then(|a| a.get("name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let image = attrs.and_then(|a| a.get("image")).and_then(|i| i.get("original")).and_then(|v| v.as_str()).map(String::from);
                Character { name, role, actor: None, image }
            })
            .collect()
    }

    fn extract_staff(&self, data: &Value, included: Option<&Vec<Value>>) -> Vec<StaffMember> {
        let included = match included { Some(i) => i, None => return vec![] };

        Self::extract_linked_people(data, included, "staff", "mediaStaff", "people", "person")
            .into_iter()
            .take(8)
            .map(|(role, node)| {
                let attrs = node.get("attributes");
                let name  = attrs.and_then(|a| a.get("name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let image = attrs.and_then(|a| a.get("image")).and_then(|i| i.get("original")).and_then(|v| v.as_str()).map(String::from);
                StaffMember { name, role, image }
            })
            .collect()
    }

    fn extract_studio(&self, data: &Value, included: Option<&Vec<Value>>) -> Option<String> {
        let included = included?;
        let prod_ids = Self::rel_ids(data, "productions");

        for prod in Self::find_included(included, "mediaProductions", &prod_ids) {
            let role = prod.get("attributes").and_then(|a| a.get("role")).and_then(|v| v.as_str()).unwrap_or("");
            if role != "studio" { continue; }

            let company_id = prod
                .get("relationships").and_then(|r| r.get("company"))
                .and_then(|c| c.get("data"))
                .and_then(|d| d.get("id"))
                .and_then(|v| v.as_str())?;

            if let Some(comp) = Self::find_one_included(included, "producers", company_id) {
                return comp.get("attributes").and_then(|a| a.get("name")).and_then(|v| v.as_str()).map(String::from);
            }
        }
        None
    }

    fn extract_cross_ids(&self, data: &Value, included: Option<&Vec<Value>>) -> HashMap<String, String> {
        let mut ids = HashMap::new();
        let included = match included { Some(i) => i, None => return ids };

        let mapping_ids = Self::rel_ids(data, "mappings");
        for item in Self::find_included(included, "mappings", &mapping_ids) {
            if let Some(attrs) = item.get("attributes") {
                if let (Some(site), Some(ext_id)) = (
                    attrs.get("externalSite").and_then(|v| v.as_str()),
                    attrs.get("externalId").and_then(|v| v.as_str()),
                ) {
                    let site_key = site.split('/').next().unwrap_or(site).to_string();
                    ids.insert(site_key, ext_id.to_string());
                }
            }
        }
        ids
    }

    fn extract_relations(&self, data: &Value, included: Option<&Vec<Value>>) -> Vec<TrackerRelation> {
        let included = match included { Some(i) => i, None => return vec![] };
        let rel_ids  = Self::rel_ids(data, "mediaRelationships");
        let mut relations = Vec::new();

        for item in Self::find_included(included, "mediaRelationships", &rel_ids) {
            let relation_type = item
                .get("attributes").and_then(|a| a.get("role")).and_then(|v| v.as_str())
                .unwrap_or("related")
                .to_string();

            let dest = item.get("relationships").and_then(|r| r.get("destination")).and_then(|d| d.get("data"));
            let dest_id   = dest.and_then(|d| d.get("id")).and_then(|v| v.as_str());
            let dest_type = dest.and_then(|d| d.get("type")).and_then(|v| v.as_str()).unwrap_or("anime");

            if let Some(dest_id) = dest_id {
                let media = match Self::find_one_included(included, dest_type, dest_id) {
                    Some(dest_node) => self.media_from_data(dest_node, None),
                    None => {
                        let c_type = if dest_type == "manga" { ContentType::Manga } else { ContentType::Anime };
                        Some(TrackerMedia {
                            tracker_id:       dest_id.to_string(),
                            tracker_url:      None,
                            cross_ids:        HashMap::new(),
                            content_type:     c_type,
                            title:            String::new(),
                            alt_titles:       vec![],
                            title_i18n:       Default::default(),
                            synopsis:         None,
                            cover_image:      None,
                            banner_image:     None,
                            episode_count:    None,
                            chapter_count:    None,
                            status:           None,
                            genres:           vec![],
                            tags:             vec![],
                            nsfw:             false,
                            release_date:     None,
                            end_date:         None,
                            rating:           None,
                            trailer_url:      None,
                            format:           None,
                            studio:           None,
                            characters:       vec![],
                            staff:            vec![],
                            relations:        vec![],
                            episode_duration: None,
                        })
                    }
                };

                if let Some(media) = media {
                    relations.push(TrackerRelation { relation_type, media });
                }
            }
        }

        relations
    }

    fn library_entry_to_user_list(&self, entry: &Value, included: &Vec<Value>) -> Option<UserListEntry> {
        let attrs = entry.get("attributes")?;

        let media_rel  = entry.get("relationships").and_then(|r| r.get("media")).and_then(|m| m.get("data"))?;
        let media_id   = media_rel.get("id")?.as_str()?;
        let media_type = media_rel.get("type")?.as_str()?;

        let media_obj  = Self::find_one_included(included, media_type, media_id)?;
        let media_attrs = media_obj.get("attributes")?;
        let tracker_media = self.media_from_data(media_obj, Some(included));

        let title = media_attrs.get("canonicalTitle").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
        let poster = media_attrs
            .get("posterImage")
            .and_then(|p| p.get("medium").or(p.get("small")))
            .and_then(|v| v.as_str())
            .map(String::from);

        let content_type = match media_type {
            "anime" => ContentType::Anime,
            "manga" => {
                let subtype = media_attrs.get("subtype").and_then(|v| v.as_str()).unwrap_or("");
                if matches!(subtype, "novel" | "light_novel") { ContentType::Novel } else { ContentType::Manga }
            }
            _ => ContentType::Anime,
        };

        let status = attrs.get("status").and_then(|v| v.as_str()).map(|s| match s {
            "current"   => "CURRENT",
            "planned"   => "PLANNING",
            "completed" => "COMPLETED",
            "on_hold"   => "PAUSED",
            "dropped"   => "DROPPED",
            other       => other,
        }.to_string());

        let score = attrs
            .get("ratingTwenty").and_then(|v| v.as_i64()).map(|r| r as f64 / 2.0)
            .or_else(|| {
                attrs.get("rating").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|r| r * 2.0)
            });

        let take_date = |key: &str| -> Option<String> {
            attrs.get(key).and_then(|v| v.as_str())
                .map(|s| s.get(..10).unwrap_or(s).to_string())
        };

        Some(UserListEntry {
            tracker_media_id: media_id.to_string(),
            title,
            poster,
            content_type,
            format:        media_attrs.get("subtype").and_then(|v| v.as_str()).map(String::from),
            status,
            progress:      attrs.get("progress").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            score,
            start_date:    take_date("startedAt"),
            end_date:      take_date("finishedAt"),
            repeat_count:  attrs.get("reconsumeCount").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            notes:         attrs.get("notes").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from),
            is_private:    attrs.get("private").and_then(|v| v.as_bool()).unwrap_or(false),
            total_episodes: media_attrs.get("episodeCount").and_then(|v| v.as_i64()).map(|i| i as i32),
            total_chapters: media_attrs.get("chapterCount").and_then(|v| v.as_i64()).map(|i| i as i32),
            media:         tracker_media,
        })
    }

    async fn fetch_all_library(&self, token: &str, user_id: &str, kind: &str) -> CoreResult<Vec<UserListEntry>> {
        let mut entries = Vec::new();
        let include = "media,media.categories,media.mappings";
        let mut offset = 0usize;
        let limit = 500;

        loop {
            let path = format!(
                "/library-entries?filter[userId]={}&filter[kind]={}&include={}&page[limit]={}&page[offset]={}",
                user_id, kind, include, limit, offset
            );

            let res = self.get_auth(&path, token).await?;
            let data = match res.get("data").and_then(|d| d.as_array()) {
                Some(d) if !d.is_empty() => d.clone(),
                _ => break,
            };

            let included = res.get("included").and_then(|i| i.as_array()).cloned().unwrap_or_default();

            for entry in &data {
                if let Some(ule) = self.library_entry_to_user_list(entry, &included) {
                    entries.push(ule);
                }
            }

            if data.len() < limit { break; }
            offset += limit;
        }

        Ok(entries)
    }

    fn normalize_status(s: &str) -> Status {
        match s {
            "finished"  => Status::Completed,
            "current"   => Status::Ongoing,
            "upcoming"  => Status::Planned,
            "tba"       => Status::Planned,
            "unreleased" => Status::Planned,
            _           => Status::Ongoing,
        }
    }
}

#[async_trait]
impl TrackerProvider for KitsuProvider {
    fn name(&self) -> &'static str { "kitsu" }
    fn display_name(&self) -> &'static str { "Kitsu" }
    fn icon_url(&self) -> &'static str { "https://kitsu.io/favicon.ico" }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga, ContentType::Novel]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "password".to_string(),
            auth_url:   OAUTH_URL.to_string(),
            token_url:  Some(OAUTH_URL.to_string()),
            client_id:  Some(CLIENT_ID.to_string()),
            scopes:     vec![],
        }
    }

    async fn validate_and_store_token(&self, access_token: &str, token_type: &str) -> CoreResult<TokenData> {
        let res  = self.get_auth("/users?filter[self]=true", access_token).await?;
        let user = res.get("data").and_then(|d| d.as_array()).and_then(|arr| arr.first())
            .ok_or_else(|| CoreError::AuthError("error.tracker.invalid_credentials".into()))?;

        let attrs = user.get("attributes");

        Ok(TokenData {
            access_token:    access_token.to_string(),
            refresh_token:   None,
            token_type:      token_type.to_string(),
            expires_at:      Utc::now()
                .checked_add_signed(chrono::Duration::days(30))
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
            tracker_user_id: user.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            display_name:    attrs.and_then(|a| a.get("name")).and_then(|v| v.as_str()).map(str::to_string),
            avatar_url:      attrs.and_then(|a| a.get("avatar")).and_then(|a| a.get("original")).and_then(|v| v.as_str()).map(str::to_string),
            profile_url:     None,
            score_format:    None,
        })
    }

    async fn search(
        &self,
        query: Option<&str>,
        content_type: ContentType,
        limit: usize,
        page: usize,
        sort: Option<&str>,
        genre: Option<&str>,
        format: Option<&str>,
        _nsfw: Option<bool>,
        status: Option<&str>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let endpoint   = match content_type { ContentType::Anime => "anime", _ => "manga" };
        let page_limit = limit.min(20);
        let offset     = (page.max(1) - 1) * page_limit;

        let mut path = format!(
            "/{endpoint}?page[limit]={page_limit}&page[offset]={offset}\
             &include=categories,mappings,mediaRelationships"
        );

        if let Some(q) = query.filter(|q| !q.trim().is_empty()) {
            path.push_str(&format!("&filter[text]={}", urlencoding::encode(q)));
        }
        if let Some(s) = sort   { path.push_str(&format!("&sort={}", s)); }
        if let Some(g) = genre  { path.push_str(&format!("&filter[categories]={}", g)); }
        if let Some(f) = format { path.push_str(&format!("&filter[subtype]={}", f)); }
        if let Some(s) = status {
            let kitsu_status = match s {
                "completed" => "finished",
                "ongoing"   => "current",
                "upcoming"  => "upcoming",
                other       => other,
            };
            path.push_str(&format!("&filter[status]={}", kitsu_status));
        }

        let res = self.get_public(&path).await?;
        let data = res.get("data").and_then(|d| d.as_array())
            .ok_or_else(|| CoreError::NotFound("Kitsu search: no data".into()))?;
        let included = res.get("included").and_then(|i| i.as_array()).cloned().unwrap_or_default();

        Ok(data.iter().filter_map(|item| self.media_from_data(item, Some(&included))).collect())
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let (endpoint, id) = if let Some(s) = tracker_id.strip_prefix("anime:") {
            ("anime", s)
        } else if let Some(s) = tracker_id.strip_prefix("manga:") {
            ("manga", s)
        } else {
            ("anime", tracker_id)
        };

        let path = format!(
            "/{}/{}?include=categories,mappings,mediaRelationships,\
             characters.character,staff.person,productions.company",
            endpoint, id
        );

        match self.get_public(&path).await {
            Ok(res) => {
                let included = res.get("included").and_then(|i| i.as_array()).cloned().unwrap_or_default();
                Ok(res.get("data").and_then(|d| self.media_from_data(d, Some(&included))))
            }
            Err(CoreError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        let (r_trending_anime, r_top_anime, r_trending_manga) = tokio::join!(
            self.get_public("/anime?sort=-popularityRank&page[limit]=20&include=categories"),
            self.get_public("/anime?sort=-averageRating&page[limit]=20&include=categories"),
            self.get_public("/manga?sort=-popularityRank&page[limit]=20&include=categories"),
        );

        let mut home = HashMap::new();

        let mut insert_section = |label: &str, result: CoreResult<Value>| {
            if let Ok(res) = result {
                let included = res.get("included").and_then(|i| i.as_array()).cloned().unwrap_or_default();
                if let Some(data) = res.get("data").and_then(|d| d.as_array()) {
                    home.insert(
                        label.to_string(),
                        data.iter().filter_map(|i| self.media_from_data(i, Some(&included))).collect(),
                    );
                }
            }
        };

        insert_section("Trending Anime", r_trending_anime);
        insert_section("Top Rated Anime", r_top_anime);
        insert_section("Trending Manga", r_trending_manga);

        Ok(home)
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        tracker_user_id: &str,
        _score_format: Option<&str>,
    ) -> CoreResult<Vec<UserListEntry>> {
        let (anime, manga) = tokio::try_join!(
            self.fetch_all_library(access_token, tracker_user_id, "anime"),
            self.fetch_all_library(access_token, tracker_user_id, "manga"),
        )?;

        let mut all = anime;
        all.extend(manga);
        Ok(all)
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let user_res = self.get_auth("/users?filter[self]=true", access_token).await?;
        let user_id = user_res
            .get("data").and_then(|d| d.as_array()).and_then(|arr| arr.first())
            .and_then(|u| u.get("id")).and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::AuthError("error.tracker.invalid_credentials".into()))?
            .to_string();

        let (media_type, raw_id) = if let Some(s) = params.media_id.strip_prefix("anime:") {
            ("anime", s.to_string())
        } else if let Some(s) = params.media_id.strip_prefix("manga:") {
            ("manga", s.to_string())
        } else {
            ("anime", params.media_id.clone())
        };

        let existing_path = format!(
            "/library-entries?filter[userId]={}&filter[mediaId]={}&filter[kind]={}",
            user_id, raw_id, media_type
        );
        let existing  = self.get_auth(&existing_path, access_token).await?;
        let entry_id  = existing
            .get("data").and_then(|d| d.as_array()).and_then(|arr| arr.first())
            .and_then(|e| e.get("id")).and_then(|v| v.as_str())
            .map(String::from);

        let kitsu_status = params.status.as_deref().map(|s| match s {
            "CURRENT"   => "current",
            "PLANNING"  => "planned",
            "COMPLETED" => "completed",
            "PAUSED"    => "on_hold",
            "DROPPED"   => "dropped",
            other       => other,
        });

        let rating_twenty = params.score.map(|s| (s * 2.0).round().clamp(2.0, 20.0) as i64);

        let mut entry_attrs = json!({});
        if let Some(st)     = kitsu_status          { entry_attrs["status"]         = json!(st); }
        if let Some(prog)   = params.progress        { entry_attrs["progress"]       = json!(prog); }
        if let Some(r)      = rating_twenty          { entry_attrs["ratingTwenty"]   = json!(r); }
        if let Some(repeat) = params.repeat_count    { entry_attrs["reconsumeCount"] = json!(repeat); }
        if let Some(notes)  = params.notes           { entry_attrs["notes"]          = json!(notes); }
        if let Some(priv_)  = params.is_private      { entry_attrs["private"]        = json!(priv_); }

        if let Some(eid) = entry_id {
            self.mutate(
                reqwest::Method::PATCH,
                &format!("/library-entries/{}", eid),
                access_token,
                Some(&json!({ "data": { "id": eid, "type": "libraryEntries", "attributes": entry_attrs } })),
            ).await?;
        } else {
            self.mutate(
                reqwest::Method::POST,
                "/library-entries",
                access_token,
                Some(&json!({
                    "data": {
                        "type": "libraryEntries",
                        "attributes": entry_attrs,
                        "relationships": {
                            "user":  { "data": { "id": user_id, "type": "users" } },
                            "media": { "data": { "id": raw_id,  "type": media_type } }
                        }
                    }
                })),
            ).await?;
        }

        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        match self.mutate(reqwest::Method::DELETE, &format!("/library-entries/{}", media_id), access_token, None).await {
            Ok(_)                        => Ok(true),
            Err(CoreError::NotFound(_))  => Ok(false),
            Err(e)                       => Err(e),
        }
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata {
        let now   = Utc::now().timestamp();
        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _                  => media.chapter_count.unwrap_or(0),
        };

        Metadata {
            id:               None,
            cid:              cid.to_string(),
            source_name:      self.name().to_string(),
            source_id:        Some(media.tracker_id.clone()),
            subtype:          media.format.clone(),
            title:            media.title.clone(),
            alt_titles:       media.alt_titles.clone(),
            title_i18n:       media.title_i18n.clone(),
            synopsis:         media.synopsis.clone(),
            cover_image:      media.cover_image.clone(),
            banner_image:     media.banner_image.clone(),
            eps_or_chapters:  EpisodeData::Count(count),
            status:           media.status.as_deref().map(Self::normalize_status),
            genres:           media.genres.clone(),
            release_date:     media.release_date.clone(),
            end_date:         media.end_date.clone(),
            rating:           media.rating,
            trailer_url:      media.trailer_url.clone(),
            characters:       media.characters.clone(),
            studio:           media.studio.clone(),
            staff:            media.staff.clone(),
            external_ids:     json!({}),
            episode_duration: media.episode_duration,
            created_at:       now,
            updated_at:       now,
        }
    }
}