use async_trait::async_trait;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration as StdDuration;

use crate::content::models::{Character, ContentType, EpisodeData, Metadata, StaffMember, Status};
use crate::error::{CoreError, CoreResult};
use crate::schedule::types::AiringEpisode;

pub(crate) use super::{TokenData, TrackerMedia, TrackerProvider, UpdateEntryParams, UserListEntry};

const MEDIA_FRAGMENT: &str = r#"
fragment mediaFields on Media {
  id idMal type format
  title { romaji english native userPreferred }
  synonyms
  description bannerImage
  coverImage { extraLarge large }
  episodes chapters duration
  status
  startDate { year month day }
  endDate   { year month day }
  genres
  tags { name isMediaSpoiler isAdult }
  isAdult
  averageScore meanScore
  trailer { id site }
  relations {
  edges {
    relationType(version: 2)
    node {
      id idMal
      title { romaji english native userPreferred }
      description
      type status format
      coverImage { large }
      episodes chapters
      averageScore
      startDate { year month day }
      genres
    }
  }
}
  characters(role: MAIN, perPage: 6) {
    edges {
      role
      node {
        id
        name { full }
        image { large }
      }
      voiceActors(language: JAPANESE, sort: [RELEVANCE, ID]) {
        id
        name { full }
        image { large }
      }
    }
  }
  staff(perPage: 8) {
    edges {
      role
      node {
        id
        name { full }
        image { large }
      }
    }
  }
  studios(isMain: true) { nodes { name } }
}
"#;

const GLOBAL_AIRING_QUERY: &str = r#"
query ($from: Int, $to: Int, $page: Int) {
  Page(page: $page, perPage: 50) {
    pageInfo { hasNextPage }
    airingSchedules(airingAt_greater: $from, airingAt_lesser: $to, sort: TIME) {
      episode
      airingAt
      media { ...mediaFields }
    }
  }
}
"#;

const SEARCH_QUERY: &str = r#"
query ($search: String, $page: Int, $perPage: Int, $type: MediaType,
       $sort: [MediaSort], $status: MediaStatus, $genre: String, $format: MediaFormat,
       $formatIn: [MediaFormat], $formatNotIn: [MediaFormat], $isAdult: Boolean) {
  Page(page: $page, perPage: $perPage) {
    media(search: $search, type: $type, sort: $sort, status: $status,
          genre: $genre, format: $format, format_in: $formatIn, format_not_in: $formatNotIn, isAdult: $isAdult) {
      ...mediaFields
    }
  }
}
"#;

const HOME_QUERY_ANIME_1: &str = r#"
query {
  trending_anime: Page(perPage: 20) {
    media(sort: TRENDING_DESC, type: ANIME, isAdult: false) { ...mediaFields }
  }
  popular_anime: Page(perPage: 20) {
    media(sort: POPULARITY_DESC, type: ANIME, isAdult: false) { ...mediaFields }
  }
  top_rated_anime: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: ANIME, isAdult: false) { ...mediaFields }
  }
}
"#;

const HOME_QUERY_ANIME_2: &str = r#"
query {
  seasonal_anime: Page(perPage: 20) {
    media(sort: POPULARITY_DESC, status: RELEASING, type: ANIME, isAdult: false) { ...mediaFields }
  }
  upcoming_anime: Page(perPage: 20) {
    media(sort: POPULARITY_DESC, status: NOT_YET_RELEASED, type: ANIME, isAdult: false) { ...mediaFields }
  }
  recently_finished_anime: Page(perPage: 20) {
    media(sort: END_DATE_DESC, status: FINISHED, type: ANIME, isAdult: false) { ...mediaFields }
  }
  top_action_anime: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: ANIME, genre: "Action", isAdult: false) { ...mediaFields }
  }
}
"#;

const HOME_QUERY_ANIME_3: &str = r#"
query {
  top_romance_anime: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: ANIME, genre: "Romance", isAdult: false) { ...mediaFields }
  }
  top_fantasy_anime: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: ANIME, genre: "Fantasy", isAdult: false) { ...mediaFields }
  }
  top_scifi_anime: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: ANIME, genre: "Sci-Fi", isAdult: false) { ...mediaFields }
  }
  top_sports_anime: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: ANIME, genre: "Sports", isAdult: false) { ...mediaFields }
  }
}
"#;

const HOME_QUERY_MANGA_1: &str = r#"
query {
  trending_manga: Page(perPage: 20) {
    media(sort: TRENDING_DESC, type: MANGA, format_not_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
  popular_manga: Page(perPage: 20) {
    media(sort: POPULARITY_DESC, type: MANGA, format_not_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
  top_rated_manga: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: MANGA, format_not_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
}
"#;

const HOME_QUERY_MANGA_2: &str = r#"
query {
  seasonal_manga: Page(perPage: 20) {
    media(sort: POPULARITY_DESC, status: RELEASING, type: MANGA, format_not_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
  recently_finished_manga: Page(perPage: 20) {
    media(sort: END_DATE_DESC, status: FINISHED, type: MANGA, format_not_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
}
"#;

const HOME_QUERY_MANGA_3: &str = r#"
query {
  trending_novel: Page(perPage: 20) {
    media(sort: TRENDING_DESC, type: MANGA, format_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
  popular_novel: Page(perPage: 20) {
    media(sort: POPULARITY_DESC, type: MANGA, format_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
  top_rated_novel: Page(perPage: 20) {
    media(sort: SCORE_DESC, type: MANGA, format_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
  recently_finished_novel: Page(perPage: 20) {
    media(sort: END_DATE_DESC, status: FINISHED, type: MANGA, format_in: [NOVEL], isAdult: false) { ...mediaFields }
  }
}
"#;

const USER_LIST_QUERY: &str = r#"
query ($userId: Int) {
  anime: MediaListCollection(userId: $userId, type: ANIME) {
    lists {
      entries {
        media {
          ...mediaFields
          nextAiringEpisode { episode }
        }
        status progress score repeat notes private
        startedAt   { year month day }
        completedAt { year month day }
      }
    }
  }
  manga: MediaListCollection(userId: $userId, type: MANGA) {
    lists {
      entries {
        media {
          ...mediaFields
        }
        status progress score repeat notes private
        startedAt   { year month day }
        completedAt { year month day }
      }
    }
  }
}
"#;

pub struct AniListProvider {
    client: Client,
}

impl AniListProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    const BASE_URL: &'static str = "https://graphql.anilist.co";
    const MAX_RETRIES: u32 = 3;

    async fn graphql(&self, token: Option<&str>, body: &Value) -> CoreResult<Value> {
        let mut req = self.client.post(Self::BASE_URL)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        let req = req.json(body);
        let res = Self::with_retry(req).await
            .map_err(|_| CoreError::Internal("error.tracker.auth_network_error".into()))?;

        if !res.status().is_success() {
            println!("{:?}", res.text().await);
            return Err(CoreError::Internal("error.tracker.token_exchange_failed".into()));
        }

        let json: Value = res.json().await
            .map_err(|_e| CoreError::Internal("error.tracker.token_exchange_failed".into()))?;

        if let Some(errors) = json.get("errors").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                if errors[0].get("status").and_then(|s| s.as_i64()) == Some(404) {
                    return Ok(json);
                }
                return Err(CoreError::Internal("error.tracker.token_exchange_failed".into()));
            }
        }

        Ok(json)
    }

    async fn with_retry(request_builder: reqwest::RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
        let mut delay = StdDuration::from_secs(1);
        let mut last_err = None;

        for attempt in 0..=Self::MAX_RETRIES {
            match request_builder.try_clone().unwrap().send().await {
                Ok(res) => {
                    if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        let wait = res.headers()
                            .get("Retry-After")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(StdDuration::from_secs)
                            .unwrap_or(delay);

                        if attempt < Self::MAX_RETRIES {
                            tracing::warn!("AniList rate limit. Waiting {}s…", wait.as_secs());
                            tokio::time::sleep(wait).await;
                            delay = delay.min(StdDuration::from_secs(60)) * 2;
                            continue;
                        }
                    }
                    if res.status().is_server_error() && attempt < Self::MAX_RETRIES {
                        tokio::time::sleep(delay).await;
                        delay = delay.min(StdDuration::from_secs(30)) * 2;
                        continue;
                    }
                    return Ok(res);
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt < Self::MAX_RETRIES {
                        tokio::time::sleep(delay).await;
                        delay = delay.min(StdDuration::from_secs(30)) * 2;
                    }
                }
            }
        }
        Err(last_err.unwrap())
    }

    fn parse_date(obj: &Value) -> Option<String> {
        let y = obj.get("year").and_then(|v| v.as_i64())?;
        let m = obj.get("month").and_then(|v| v.as_i64()).unwrap_or(1);
        let d = obj.get("day").and_then(|v| v.as_i64()).unwrap_or(1);
        Some(format!("{:04}-{:02}-{:02}", y, m, d))
    }

    fn to_fuzzy_date(date_str: Option<&str>) -> Value {
        if let Some(s) = date_str {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 3 {
                return json!({
                    "year":  parts[0].parse::<i32>().unwrap_or(0),
                    "month": parts[1].parse::<i32>().unwrap_or(0),
                    "day":   parts[2].parse::<i32>().unwrap_or(0),
                });
            }
        }
        json!(null)
    }

    fn media_to_tracker_media(&self, data: &Value) -> Option<TrackerMedia> {
        let tracker_id = data.get("id")?.as_i64()?.to_string();

        let format_str = data.get("format").and_then(|v| v.as_str());

        let content_type = match data.get("type").and_then(|v| v.as_str()) {
            Some("MANGA") if matches!(format_str, Some("NOVEL") | Some("LIGHT_NOVEL")) => ContentType::Novel,
            Some("MANGA") => ContentType::Manga,
            _ => ContentType::Anime,
        };

        let mut cross_ids = HashMap::new();
        if let Some(mal_id) = data.get("idMal").and_then(|v| v.as_i64()) {
            let prefix = match content_type {
                ContentType::Anime => "anime",
                ContentType::Manga | ContentType::Novel => "manga",
            };

            cross_ids.insert(
                "mal".to_string(),
                format!("{}:{}", prefix, mal_id),
            );
        }

        let titles_obj = data.get("title");

        let title = titles_obj
            .and_then(|t| t.get("romaji").or(t.get("english")))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut title_i18n: HashMap<String, String> = HashMap::new();
        if let Some(t) = titles_obj {
            if let Some(s) = t.get("native").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                title_i18n.insert("native".to_string(), s.to_string());
            }
            if let Some(s) = t.get("romaji").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                title_i18n.insert("romaji".to_string(), s.to_string());
            }
            if let Some(s) = t.get("english").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                title_i18n.insert("english".to_string(), s.to_string());
            }
        }

        let mut alt_titles = vec![];
        if let Some(t) = titles_obj.and_then(|t| t.get("english")).and_then(|v| v.as_str()) {
            alt_titles.push(t.to_string());
        }
        if let Some(t) = titles_obj.and_then(|t| t.get("native")).and_then(|v| v.as_str()) {
            alt_titles.push(t.to_string());
        }
        if let Some(syns) = data.get("synonyms").and_then(|v| v.as_array()) {
            alt_titles.extend(syns.iter().filter_map(|v| v.as_str().map(String::from)));
        }

        let cover_image = data.get("coverImage")
            .and_then(|i| i.get("extraLarge").or(i.get("large")))
            .and_then(|v| v.as_str())
            .map(String::from);

        let genres: Vec<String> = data.get("genres").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let tags: Vec<String> = data.get("tags").and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter(|t| !t.get("isAdult").and_then(|v| v.as_bool()).unwrap_or(false))
                .filter_map(|t| t.get("name").and_then(|v| v.as_str()).map(String::from))
                .collect())
            .unwrap_or_default();

        let trailer_url = data.get("trailer").and_then(|t| {
            let site = t.get("site").and_then(|s| s.as_str())?;
            let id   = t.get("id").and_then(|s| s.as_str())?;
            match site {
                "youtube"     => Some(format!("https://www.youtube.com/watch?v={}", id)),
                "dailymotion" => Some(format!("https://www.dailymotion.com/video/{}", id)),
                _ => None,
            }
        });

        let rating = data.get("averageScore").or(data.get("meanScore"))
            .and_then(|v| v.as_f64())
            .map(|v| (v / 10.0) as f32);

        let studio = data.get("studios")
            .and_then(|s| s.get("nodes"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|n| n.get("name").and_then(|v| v.as_str()).map(String::from));

        let mut characters = Vec::new();
        if let Some(edges) = data.get("characters").and_then(|c| c.get("edges")).and_then(|e| e.as_array()) {
            for edge in edges {
                let role  = edge.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let node  = edge.get("node");
                let name  = node.and_then(|n| n.get("name")).and_then(|n| n.get("full")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let image = node.and_then(|n| n.get("image")).and_then(|i| i.get("large")).and_then(|v| v.as_str()).map(String::from);
                let actor = edge.get("voiceActors").and_then(|v| v.as_array()).and_then(|arr| arr.first())
                    .and_then(|va| va.get("name")).and_then(|n| n.get("full")).and_then(|v| v.as_str()).map(String::from);
                characters.push(Character { name, role, actor, image });
            }
        }

        let mut staff = Vec::new();
        if let Some(edges) = data.get("staff").and_then(|s| s.get("edges")).and_then(|e| e.as_array()) {
            for edge in edges {
                let role  = edge.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let node  = edge.get("node");
                let name  = node.and_then(|n| n.get("name")).and_then(|n| n.get("full")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let image = node.and_then(|n| n.get("image")).and_then(|i| i.get("large")).and_then(|v| v.as_str()).map(String::from);
                staff.push(StaffMember { name, role, image });
            }
        }

        let mut relations = Vec::new();
        if let Some(edges) = data.get("relations").and_then(|r| r.get("edges")).and_then(|e| e.as_array()) {
            for edge in edges {
                let relation_type = edge.get("relationType").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if let Some(node) = edge.get("node") {
                    if let Some(rel_media) = self.media_to_tracker_media(node) {
                        relations.push(super::TrackerRelation { relation_type, media: rel_media });
                    }
                }
            }
        }

        Some(TrackerMedia {
            tracker_id,
            tracker_url: None,
            cross_ids,
            content_type,
            title,
            alt_titles,
            title_i18n,
            synopsis:      data.get("description").and_then(|v| v.as_str()).map(String::from),
            cover_image,
            banner_image:  data.get("bannerImage").and_then(|v| v.as_str()).map(String::from),
            episode_count: data.get("episodes").and_then(|v| v.as_i64()).map(|i| i as i32),
            chapter_count: data.get("chapters").and_then(|v| v.as_i64()).map(|i| i as i32),
            episode_duration: data.get("duration").and_then(|v| v.as_i64()).map(|i| i as i32),
            status:        data.get("status").and_then(|v| v.as_str()).map(String::from),
            genres,
            tags,
            nsfw:          data.get("isAdult").and_then(|v| v.as_bool()).unwrap_or(false),
            release_date:  data.get("startDate").and_then(Self::parse_date),
            end_date:      data.get("endDate").and_then(Self::parse_date),
            rating,
            trailer_url,
            format:        format_str.map(String::from),
            studio,
            characters,
            staff,
            relations,
        })
    }

    fn normalize_status(s: &str) -> Status {
        match s {
            "FINISHED"         => Status::Completed,
            "RELEASING"        => Status::Ongoing,
            "NOT_YET_RELEASED" => Status::Planned,
            "CANCELLED"        => Status::Cancelled,
            "HIATUS"           => Status::Hiatus,
            _                  => Status::Ongoing,
        }
    }
}

#[async_trait]
impl TrackerProvider for AniListProvider {
    fn name(&self) -> &'static str { "anilist" }
    fn display_name(&self) -> &'static str { "AniList" }
    fn icon_url(&self) -> &'static str {
        "https://anilist.co/img/icons/android-chrome-512x512.png"
    }
    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga, ContentType::Novel]
    }
    fn auth_config(&self) -> super::TrackerAuthConfig {
        super::TrackerAuthConfig {
            oauth_flow: "implicit".into(),
            auth_url:   "https://anilist.co/api/v2/oauth/authorize".into(),
            token_url: None,
            client_id:  Some("37027".into()),
            scopes:     vec![],
        }
    }

    async fn refresh_score_format(&self, access_token: &str) -> CoreResult<Option<String>> {
        let res = self.graphql(Some(access_token), &json!({
        "query": "query { Viewer { mediaListOptions { scoreFormat } } }"
    })).await?;

        Ok(res.get("data")
            .and_then(|d| d.get("Viewer"))
            .and_then(|v| v.get("mediaListOptions"))
            .and_then(|o| o.get("scoreFormat"))
            .and_then(|v| v.as_str())
            .map(str::to_string))
    }

    async fn validate_and_store_token(&self, access_token: &str, token_type: &str) -> CoreResult<TokenData> {
        let res = self.graphql(Some(access_token), &json!({
            "query": "query { Viewer { id name avatar { large } siteUrl mediaListOptions { scoreFormat } } }"
        })).await?;

        let viewer = res.get("data").and_then(|d| d.get("Viewer"))
            .ok_or_else(|| CoreError::AuthError("error.tracker.invalid_credentials".into()))?;

        let score_format = viewer
            .get("mediaListOptions")
            .and_then(|o| o.get("scoreFormat"))
            .and_then(|v| v.as_str())
            .unwrap_or("POINT_10")
            .to_string();

        let user_id = viewer.get("id").and_then(|v| v.as_i64())
            .ok_or_else(|| CoreError::AuthError("error.tracker.invalid_credentials".into()))?;

        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(365))
            .ok_or_else(|| CoreError::Internal("Date overflow".into()))?
            .to_rfc3339();

        Ok(TokenData {
            access_token:    access_token.to_string(),
            refresh_token:   None,
            token_type:      token_type.to_string(),
            expires_at,
            tracker_user_id: user_id.to_string(),
            display_name: viewer.get("name").and_then(|v| v.as_str()).map(str::to_string),
            avatar_url:   viewer.get("avatar").and_then(|a| a.get("large")).and_then(|v| v.as_str()).map(str::to_string),
            profile_url:  viewer.get("siteUrl").and_then(|v| v.as_str()).map(str::to_string),
            score_format: Some(score_format)
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
        nsfw: Option<bool>,
        status: Option<&str>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let al_type = match content_type {
            ContentType::Manga | ContentType::Novel => "MANGA",
            _ => "ANIME",
        };

        let al_status = status.map(|s| match s {
            "completed"  => "FINISHED",
            "ongoing"    => "RELEASING",
            "upcoming"   => "NOT_YET_RELEASED",
            "cancelled"  => "CANCELLED",
            "hiatus"     => "HIATUS",
            v            => v,
        });

        let mut variables = json!({
        "page":    page.max(1),
        "perPage": limit.min(50),
        "type":    al_type,
        "isAdult": nsfw.unwrap_or(false)
    });

        match content_type {
            ContentType::Novel => { variables["formatIn"] = json!(["NOVEL"]); }
            ContentType::Manga => { variables["formatNotIn"] = json!(["NOVEL"]); }
            _ => {}
        }

        if let Some(q) = query.filter(|q| !q.trim().is_empty()) { variables["search"] = json!(q); }
        if let Some(s) = sort      { variables["sort"]   = json!([s]); }
        if let Some(g) = genre     { variables["genre"]  = json!(g); }
        if let Some(f) = format    { variables["format"] = json!(f); }
        if let Some(s) = al_status { variables["status"] = json!(s); }

        let full_query = format!("{}\n{}", SEARCH_QUERY, MEDIA_FRAGMENT);
        let res = self.graphql(None, &json!({ "query": full_query, "variables": variables })).await?;

        let media_list = res.get("data")
            .and_then(|d| d.get("Page"))
            .and_then(|p| p.get("media"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| CoreError::NotFound("No results from AniList".into()))?;

        Ok(media_list.iter().filter_map(|m| self.media_to_tracker_media(m)).collect())
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let query      = r#"query ($id: Int) { Media(id: $id) { ...mediaFields } } "#;
        let full_query = format!("{}\n{}", query, MEDIA_FRAGMENT);

        let res = self.graphql(None, &json!({
            "query":     full_query,
            "variables": { "id": tracker_id.parse::<i64>().unwrap_or(0) }
        })).await?;

        let media = res.get("data").and_then(|d| d.get("Media"));
        Ok(media.and_then(|m| self.media_to_tracker_media(m)))
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        let frag = MEDIA_FRAGMENT;

        let body_a1 = json!({ "query": format!("{}\n{}", HOME_QUERY_ANIME_1, frag) });
        let body_a2 = json!({ "query": format!("{}\n{}", HOME_QUERY_ANIME_2, frag) });
        let body_a3 = json!({ "query": format!("{}\n{}", HOME_QUERY_ANIME_3, frag) });
        let body_m1 = json!({ "query": format!("{}\n{}", HOME_QUERY_MANGA_1, frag) });
        let body_m2 = json!({ "query": format!("{}\n{}", HOME_QUERY_MANGA_2, frag) });
        let body_m3 = json!({ "query": format!("{}\n{}", HOME_QUERY_MANGA_3, frag) });

        let (r_a1, r_a2, r_a3, r_m1, r_m2, r_m3) = tokio::try_join!(
        self.graphql(None, &body_a1),
        self.graphql(None, &body_a2),
        self.graphql(None, &body_a3),
        self.graphql(None, &body_m1),
        self.graphql(None, &body_m2),
        self.graphql(None, &body_m3),
    )?;

        let mut sections = HashMap::new();

        for (res, keys) in [
            (r_a1, vec!["trending_anime", "popular_anime", "top_rated_anime"]),
            (r_a2, vec!["seasonal_anime", "upcoming_anime", "recently_finished_anime", "top_action_anime"]),
            (r_a3, vec!["top_romance_anime", "top_fantasy_anime", "top_scifi_anime", "top_sports_anime"]),
            (r_m1, vec!["trending_manga", "popular_manga", "top_rated_manga"]),
            (r_m2, vec!["seasonal_manga", "recently_finished_manga"]),
            (r_m3, vec!["trending_novel", "popular_novel", "top_rated_novel", "recently_finished_novel"]),
        ] {
            let data = res.get("data")
                .ok_or_else(|| CoreError::NotFound("AniList home: no data".into()))?;
            for key in keys {
                let items = data.get(key)
                    .and_then(|p| p.get("media"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|m| self.media_to_tracker_media(m)).collect())
                    .unwrap_or_default();
                sections.insert(key.to_string(), items);
            }
        }

        Ok(sections)
    }

    async fn get_user_list(&self, access_token: &str, tracker_user_id: &str, score_format: Option<&str>,) -> CoreResult<Vec<UserListEntry>> {
        let user_id    = tracker_user_id.parse::<i64>().unwrap_or(0);
        let full_query = format!("{}\n{}", USER_LIST_QUERY, MEDIA_FRAGMENT);
        let res = self.graphql(
            Some(access_token),
            &json!({ "query": full_query, "variables": { "userId": user_id } }),
        ).await?;

        let data = res.get("data").ok_or_else(|| CoreError::Internal("AniList list: missing data".into()))?;
        let mut results = Vec::new();

        for (media_type, al_type) in &[("anime", "ANIME"), ("manga", "MANGA")] {
            if let Some(lists) = data.get(media_type).and_then(|a| a.get("lists")).and_then(|l| l.as_array()) {
                for list in lists {
                    if let Some(entries) = list.get("entries").and_then(|e| e.as_array()) {
                        for entry in entries {
                            let media = entry.get("media");

                            let tracker_media_id = media
                                .and_then(|m| m.get("id"))
                                .and_then(|i| i.as_i64())
                                .map(|i| i.to_string())
                                .unwrap_or_default();

                            let format = media.and_then(|m| m.get("format")).and_then(|v| v.as_str());

                            let resolved_type = if *al_type == "MANGA"
                                && matches!(format, Some("LIGHT_NOVEL") | Some("NOVEL"))
                            {
                                ContentType::Novel
                            } else if *al_type == "MANGA" {
                                ContentType::Manga
                            } else {
                                ContentType::Anime
                            };

                            let episodes  = media.and_then(|m| m.get("episodes")).and_then(|i| i.as_i64());
                            let next_ep   = media.and_then(|m| m.get("nextAiringEpisode"))
                                .and_then(|n| n.get("episode")).and_then(|i| i.as_i64());
                            let total_episodes = episodes.or_else(|| next_ep.map(|e| e - 1)).map(|i| i as i32);
                            let total_chapters = media.and_then(|m| m.get("chapters")).and_then(|i| i.as_i64()).map(|i| i as i32);

                            let title = media.and_then(|m| m.get("title"))
                                .and_then(|t| t.get("userPreferred").or(t.get("english")).or(t.get("romaji")))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown").to_string();

                            let poster = media.and_then(|m| m.get("coverImage"))
                                .and_then(|c| c.get("extraLarge").or(c.get("large")))
                                .and_then(|v| v.as_str())
                                .map(String::from);

                            let tracker_media = media.and_then(|m| self.media_to_tracker_media(m));

                            results.push(UserListEntry {
                                tracker_media_id,
                                title,
                                poster,
                                content_type: resolved_type,
                                format:       format.map(String::from),
                                status:       entry.get("status").and_then(|s| s.as_str()).map(String::from),
                                progress:     entry.get("progress").and_then(|i| i.as_i64()).unwrap_or(0) as i32,
                                score:        entry.get("score").and_then(|f| f.as_f64()).map(|s| normalize_score(s, score_format.unwrap())),
                                start_date:   entry.get("startedAt").and_then(|d| Self::parse_date(d)),
                                end_date:     entry.get("completedAt").and_then(|d| Self::parse_date(d)),
                                repeat_count: entry.get("repeat").and_then(|i| i.as_i64()).unwrap_or(0) as i32,
                                notes:        entry.get("notes").and_then(|s| s.as_str()).map(String::from),
                                is_private:   entry.get("private").and_then(|b| b.as_bool()).unwrap_or(false),
                                total_episodes,
                                total_chapters,
                                media:        tracker_media,
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let mutation = r#"
            mutation ($mediaId: Int, $status: MediaListStatus, $progress: Int, $score: Float,
                      $startedAt: FuzzyDateInput, $completedAt: FuzzyDateInput,
                      $repeat: Int, $notes: String, $private: Boolean) {
                SaveMediaListEntry(
                    mediaId: $mediaId, status: $status, progress: $progress, score: $score,
                    startedAt: $startedAt, completedAt: $completedAt,
                    repeat: $repeat, notes: $notes, private: $private
                ) { id }
            }
        "#;

        let fmt = params.score_format.as_deref().unwrap_or("POINT_10");

        let media_id: i64 = params.media_id.parse().unwrap_or(0);
        let variables = json!({
            "mediaId":     media_id,
            "status":      params.status,
            "progress":    params.progress,
            "score":       params.score.map(|s| denormalize_score(s, fmt)),
            "startedAt":   Self::to_fuzzy_date(params.start_date.as_deref()),
            "completedAt": Self::to_fuzzy_date(params.end_date.as_deref()),
            "repeat":      params.repeat_count,
            "notes":       params.notes,
            "private":     params.is_private,
        });

        self.graphql(Some(access_token), &json!({ "query": mutation, "variables": variables })).await?;
        Ok(())
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let viewer = self.graphql(Some(access_token), &json!({ "query": "query { Viewer { id } }" })).await?;
        let user_id = viewer.get("data").and_then(|d| d.get("Viewer")).and_then(|v| v.get("id"))
            .ok_or_else(|| CoreError::Internal("Failed to get AniList viewer ID".into()))?;

        let mid: i64 = media_id.parse().unwrap_or(0);

        let find = self.graphql(Some(access_token), &json!({
            "query": "query ($mediaId: Int, $userId: Int) { MediaList(mediaId: $mediaId, userId: $userId) { id } }",
            "variables": { "mediaId": mid, "userId": user_id }
        })).await?;

        let list_id = find.get("data").and_then(|d| d.get("MediaList")).and_then(|l| l.get("id"))
            .ok_or_else(|| CoreError::NotFound("error.tracker.id_not_found".into()))?;

        let del = self.graphql(Some(access_token), &json!({
            "query":     "mutation ($id: Int) { DeleteMediaListEntry(id: $id) { deleted } }",
            "variables": { "id": list_id }
        })).await?;

        Ok(del.get("data")
            .and_then(|d| d.get("DeleteMediaListEntry"))
            .and_then(|x| x.get("deleted"))
            .and_then(|b| b.as_bool())
            .unwrap_or(false))
    }
    
    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata {
        let now = Utc::now().timestamp();

        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _                  => media.chapter_count.unwrap_or(0),
        };

        let status = media.status.as_deref().map(Self::normalize_status);

        Metadata {
            id:              None,
            cid:             cid.to_string(),
            source_name:     self.name().to_string(),
            source_id:       Some(media.tracker_id.clone()),
            subtype:         media.format.clone(),
            title:           media.title.clone(),
            alt_titles:      media.alt_titles.clone(),
            title_i18n:      media.title_i18n.clone(),
            synopsis:        media.synopsis.clone(),
            cover_image:     media.cover_image.clone(),
            banner_image:    media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(count),
            status,
            genres:          media.genres.clone(),
            release_date:    media.release_date.clone(),
            end_date:        media.end_date.clone(),
            rating:          media.rating,
            trailer_url:     media.trailer_url.clone(),
            characters:      media.characters.clone(),
            studio:          media.studio.clone(),
            staff:           media.staff.clone(),
            external_ids:    json!({}),
            episode_duration: media.episode_duration,
            created_at:      now,
            updated_at:      now,
        }
    }

    async fn fetch_airing_schedule(
        &self,
        from_ts: i64,
        to_ts: i64,
    ) -> CoreResult<Vec<AiringEpisode>> {
        let full_query = format!("{}\n{}", GLOBAL_AIRING_QUERY, MEDIA_FRAGMENT);
        let mut all_episodes: Vec<AiringEpisode> = Vec::new();
        let mut page = 1i32;

        loop {
            let res = self.graphql(None, &json!({
            "query":     full_query,
            "variables": { "from": from_ts, "to": to_ts, "page": page }
        })).await?;

            let page_data = res.get("data").and_then(|d| d.get("Page"));
            let has_next  = page_data
                .and_then(|p| p.get("pageInfo"))
                .and_then(|i| i.get("hasNextPage"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let schedules = match page_data
                .and_then(|p| p.get("airingSchedules"))
                .and_then(|v| v.as_array())
            {
                Some(s) if !s.is_empty() => s.clone(),
                _ => break,
            };

            for entry in &schedules {
                let episode = match entry.get("episode").and_then(|v| v.as_i64()) {
                    Some(e) => e as i32,
                    None    => continue,
                };
                let airing_at = match entry.get("airingAt").and_then(|v| v.as_i64()) {
                    Some(t) => t,
                    None    => continue,
                };
                let media = entry.get("media").and_then(|m| self.media_to_tracker_media(m));
                all_episodes.push(AiringEpisode { episode, airing_at, media });
            }

            if !has_next { break; }
            page += 1;
        }

        Ok(all_episodes)
    }
}

fn normalize_score(score: f64, format: &str) -> f64 {
    match format {
        "POINT_100"         => score / 10.0,
        "POINT_10_DECIMAL"  => score,           // already 0–10
        "POINT_10"          => score,           // already 0–10
        "POINT_5"           => score * 2.0,     // 0–5 → 0–10
        "POINT_3"           => score * (10.0 / 3.0), // 0–3 → 0–10
        _                   => score,
    }
}

fn denormalize_score(score: f64, format: &str) -> f64 {
    match format {
        "POINT_100"         => (score * 10.0).round(),
        "POINT_10_DECIMAL"  => (score * 10.0).round() / 10.0, // 1 decimal
        "POINT_10"          => score.round(),
        "POINT_5"           => (score / 2.0).round(),
        "POINT_3"           => (score / (10.0 / 3.0)).round(),
        _                   => score.round(),
    }
}