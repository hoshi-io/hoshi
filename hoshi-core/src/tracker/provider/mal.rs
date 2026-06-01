pub(crate) use super::{
    TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, TrackerRelation, UpdateEntryParams,
    UserListEntry,
};
use crate::error::{CoreError, CoreResult};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use reqwest::Client;
use crate::content::models::{Character, ContentType, EpisodeData, Metadata, StaffMember, Status};

const JIKAN_BASE_URL: &str = "https://api.jikan.moe/v4";
const MAL_API_BASE_URL: &str = "https://api.myanimelist.net/v2";
const MAL_CLIENT_ID: &str = "f3dbcf33c69b584ced3f4ee8c12d9df5";

pub struct MalProvider {
    client: Client,
}

impl MalProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn parse_media_id(id: &str) -> (&str, &str) {
        let parts: Vec<&str> = id.splitn(2, ':').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("anime", id)
        }
    }

    fn normalize_status(s: &str) -> Status {
        match s {
            "finished_airing" | "finished" => Status::Completed,
            "currently_airing" | "publishing" => Status::Ongoing,
            "not_yet_aired" | "not_yet_published" => Status::Planned,
            _ => Status::Ongoing,
        }
    }
}

#[async_trait]
impl TrackerProvider for MalProvider {
    fn name(&self) -> &'static str {
        "mal"
    }

    fn display_name(&self) -> &'static str {
        "MyAnimeList"
    }

    fn icon_url(&self) -> &'static str {
        "https://upload.wikimedia.org/wikipedia/commons/7/7a/MyAnimeList_Logo.png"
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "pkce".to_string(),
            auth_url: "https://myanimelist.net/v1/oauth2/authorize".to_string(),
            token_url: Some("https://myanimelist.net/v1/oauth2/token".to_string()),
            client_id: Some(MAL_CLIENT_ID.parse().unwrap()),
            scopes: vec![],
        }
    }

    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData> {
        let url = format!("{}/users/@me?fields=picture", MAL_API_BASE_URL);

        let res = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(CoreError::AuthError(
                "Token de MAL inválido o expirado".into(),
            ));
        }

        let user_data: MalUserResponse = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        Ok(TokenData {
            access_token: access_token.to_string(),
            refresh_token: None,
            token_type: token_type.to_string(),
            expires_at: Utc::now()
                .checked_add_signed(chrono::Duration::days(30))
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
            tracker_user_id: user_data.id.to_string(),
            display_name: user_data.name,
            avatar_url:   user_data.picture,
            profile_url:  Some(format!("https://myanimelist.net/profile/{}", user_data.id)),
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
        let endpoint = match content_type {
            ContentType::Anime => "anime",
            ContentType::Manga | ContentType::Novel => "manga",
        };

        let mut url = format!("{}/{}?limit={}&page={}", JIKAN_BASE_URL, endpoint, limit, page.max(1));

        if let Some(q) = query { url.push_str(&format!("&q={}", q)); }
        if let Some(s) = sort  { url.push_str(&format!("&order_by={}&sort=desc", s)); }
        if let Some(g) = genre { url.push_str(&format!("&genres={}", g)); }
        if let Some(f) = format { url.push_str(&format!("&type={}", f)); }
        if let Some(s) = status {
            let jikan_status = match s {
                "completed"  => "complete",
                "ongoing"    => "airing",
                "upcoming"   => "upcoming",
                s            => s,
            };
            url.push_str(&format!("&status={}", jikan_status));
        }
        if matches!(content_type, ContentType::Novel) && format.is_none() {
            url.push_str("&type=light_novel");
        }
        if nsfw.unwrap_or(false) == false { url.push_str("&sfw=true"); }

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let jikan_res: JikanSearchResponse = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        Ok(jikan_res
            .data
            .into_iter()
            .map(|item| item.into_tracker_media(content_type.clone()))
            .collect())
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let (media_type, id) = Self::parse_media_id(tracker_id);
        let url = format!("{}/{}/{}/full", JIKAN_BASE_URL, media_type, id);

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let jikan_res: JikanSingleResponse = res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let c_type = if media_type == "manga" {
            ContentType::Manga
        } else {
            ContentType::Anime
        };

        let mut tracker_media = jikan_res.data.into_tracker_media(c_type);

        let chars_url = format!("{}/{}/{}/characters", JIKAN_BASE_URL, media_type, id);
        if let Ok(c_res) = self.client.get(&chars_url).send().await {
            if let Ok(c_data) = c_res.json::<JikanCharacterResponse>().await {
                tracker_media.characters = c_data.data.into_iter().take(10).map(|c| {
                    let image = c.character.images.and_then(|i| i.jpg).and_then(|j| j.image_url);
                    let actor = c.voice_actors.unwrap_or_default().into_iter().find(|va| va.language == "Japanese").map(|va| va.person.name);
                    Character {
                        name: c.character.name,
                        role: c.role,
                        image,
                        actor,
                    }
                }).collect();
            }
        }

        if media_type == "anime" {
            let staff_url = format!("{}/{}/{}/staff", JIKAN_BASE_URL, media_type, id);
            if let Ok(s_res) = self.client.get(&staff_url).send().await {
                if let Ok(s_data) = s_res.json::<JikanStaffResponse>().await {
                    tracker_media.staff = s_data.data.into_iter().take(8).map(|s| {
                        let image = s.person.images.and_then(|i| i.jpg).and_then(|j| j.image_url);
                        StaffMember {
                            name: s.person.name,
                            role: s.positions.join(", "),
                            image,
                        }
                    }).collect();
                }
            }
        }

        Ok(Some(tracker_media))
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        let mut home = HashMap::new();

        let top_url = format!("{}/top/anime?limit=10", JIKAN_BASE_URL);
        if let Ok(res) = self.client.get(&top_url).send().await {
            if let Ok(j_res) = res.json::<JikanSearchResponse>().await {
                home.insert(
                    "Top Anime".to_string(),
                    j_res
                        .data
                        .into_iter()
                        .map(|i| i.into_tracker_media(ContentType::Anime))
                        .collect(),
                );
            }
        }

        let top_manga_url = format!("{}/top/manga?limit=10", JIKAN_BASE_URL);
        if let Ok(res) = self.client.get(&top_manga_url).send().await {
            if let Ok(j_res) = res.json::<JikanSearchResponse>().await {
                home.insert(
                    "Top Manga".to_string(),
                    j_res
                        .data
                        .into_iter()
                        .map(|i| i.into_tracker_media(ContentType::Manga))
                        .collect(),
                );
            }
        }

        Ok(home)
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        _tracker_user_id: &str,
    ) -> CoreResult<Vec<UserListEntry>> {
        let anime_url = format!(
            "{}/users/@me/animelist?fields=list_status,node.id,node.title,node.main_picture,node.alternative_titles,node.start_date,node.end_date,node.synopsis,node.mean,node.rank,node.popularity,node.num_list_users,node.num_scoring_users,node.nsfw,node.genres,node.media_type,node.status,node.num_episodes,node.start_season,node.broadcast,node.source,node.average_episode_duration,node.rating,node.studios,node.related_anime,node.related_manga,node.recommendations&limit=1000",
            MAL_API_BASE_URL
        );
        let manga_url = format!(
            "{}/users/@me/mangalist?fields=list_status,node.id,node.title,node.main_picture,node.alternative_titles,node.start_date,node.end_date,node.synopsis,node.mean,node.rank,node.popularity,node.num_list_users,node.num_scoring_users,node.nsfw,node.genres,node.media_type,node.status,node.num_volumes,node.num_chapters,node.authors{{first_name,last_name}},node.serialization,node.related_anime,node.related_manga,node.recommendations&limit=1000",
            MAL_API_BASE_URL
        );

        let mut entries: Vec<UserListEntry> = Vec::new();

        let anime_res = self
            .client
            .get(&anime_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let anime_list: MalListResponse = anime_res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        for node in anime_list.data {
            let media = node.node;
            let status = node.list_status;

            let alt_titles = {
                let mut v = Vec::new();
                if let Some(ref at) = media.alternative_titles {
                    if let Some(ref en) = at.en { if !en.is_empty() { v.push(en.clone()); } }
                    if let Some(ref ja) = at.ja { if !ja.is_empty() { v.push(ja.clone()); } }
                    if let Some(ref syns) = at.synonyms { v.extend(syns.clone()); }
                }
                v
            };

            let title_i18n = {
                let mut map = std::collections::HashMap::new();
                if let Some(ref at) = media.alternative_titles {
                    if let Some(ref en) = at.en { if !en.is_empty() { map.insert("english".to_string(), en.clone()); } }
                    if let Some(ref ja) = at.ja { if !ja.is_empty() { map.insert("native".to_string(), ja.clone()); } }
                }
                map.insert("romaji".to_string(), media.title.clone());
                map
            };

            let genres: Vec<String> = media.genres.unwrap_or_default()
                .into_iter().map(|g| g.name).collect();

            let rating_str = media.rating.as_deref().unwrap_or("").to_lowercase();
            let is_nsfw = rating_str.contains("rx") || rating_str.contains("hentai")
                || media.nsfw.as_deref() == Some("black")
                || genres.iter().any(|g| { let gl = g.to_lowercase(); gl == "hentai" || gl == "erotica" });

            let studio = media.studios
                .and_then(|mut s| s.drain(..).next().map(|s| s.name));

            let relations: Vec<TrackerRelation> = media.related_anime.unwrap_or_default()
                .into_iter()
                .map(|r| TrackerRelation {
                    relation_type: r.relation_type.to_uppercase(),
                    media: TrackerMedia {
                        tracker_id: format!("anime:{}", r.node.id),
                        tracker_url: None,
                        cross_ids: HashMap::new(),
                        content_type: ContentType::Anime,
                        title: r.node.title,
                        alt_titles: vec![],
                        title_i18n: Default::default(),
                        synopsis: None, cover_image: None, banner_image: None,
                        episode_count: None, chapter_count: None, status: None,
                        genres: vec![], tags: vec![], nsfw: false,
                        release_date: None, end_date: None, rating: None,
                        trailer_url: None, format: r.node.media_type,
                        studio: None, characters: vec![], staff: vec![], relations: vec![], episode_duration: None
                    },
                })
                .chain(
                    media.related_manga.unwrap_or_default()
                        .into_iter()
                        .map(|r| TrackerRelation {
                            relation_type: r.relation_type.to_uppercase(),
                            media: TrackerMedia {
                                tracker_id: format!("manga:{}", r.node.id),
                                tracker_url: None,
                                cross_ids: HashMap::new(),
                                content_type: ContentType::Manga,
                                title: r.node.title,
                                alt_titles: vec![],
                                title_i18n: Default::default(),
                                synopsis: None, cover_image: None, banner_image: None,
                                episode_count: None, chapter_count: None, status: None,
                                genres: vec![], tags: vec![], nsfw: false,
                                release_date: None, end_date: None, rating: None,
                                trailer_url: None, format: r.node.media_type,
                                studio: None, characters: vec![], staff: vec![], relations: vec![], episode_duration: None
                            },
                        })
                )
                .collect();

            let tracker_media = TrackerMedia {
                tracker_id: format!("anime:{}", media.id),
                tracker_url: Some(format!("https://myanimelist.net/anime/{}", media.id)),
                cross_ids: {
                    let mut m = HashMap::new();
                    m.insert("mal".to_string(), format!("anime:{}", media.id));
                    m
                },
                content_type: ContentType::Anime,
                title: media.title.clone(),
                alt_titles,
                title_i18n,
                synopsis: media.synopsis,
                cover_image: media.main_picture.as_ref().map(|p| p.large.clone().unwrap_or(p.medium.clone())),
                banner_image: None,
                episode_count: media.num_episodes,
                chapter_count: None,
                status: media.status,
                genres,
                tags: vec![],
                nsfw: is_nsfw,
                release_date: media.start_date,
                end_date: media.end_date,
                rating: media.mean,
                trailer_url: None,
                format: media.media_type,
                studio,
                characters: vec![],
                staff: vec![],
                relations,
                episode_duration: None
            };

            entries.push(UserListEntry {
                tracker_media_id: format!("anime:{}", media.id),
                title: media.title,
                poster: media.main_picture.map(|p| p.large.unwrap_or(p.medium)),
                content_type: ContentType::Anime,
                format: tracker_media.format.clone(),
                status: Some(status.status),
                progress: status.num_episodes_watched.unwrap_or(0),
                score: status.score.map(|s| s as f64),
                start_date: status.start_date,
                end_date: status.finish_date,
                repeat_count: status.num_times_rewatched.unwrap_or(0),
                notes: status.comments,
                is_private: false,
                total_episodes: media.num_episodes,
                total_chapters: None,
                media: Some(tracker_media),
            });
        }

        let manga_res = self
            .client
            .get(&manga_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let manga_list: MalListResponse = manga_res
            .json()
            .await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        for node in manga_list.data {
            let media = node.node;
            let status = node.list_status;

            let alt_titles = {
                let mut v = Vec::new();
                if let Some(ref at) = media.alternative_titles {
                    if let Some(ref en) = at.en { if !en.is_empty() { v.push(en.clone()); } }
                    if let Some(ref ja) = at.ja { if !ja.is_empty() { v.push(ja.clone()); } }
                    if let Some(ref syns) = at.synonyms { v.extend(syns.clone()); }
                }
                v
            };

            let title_i18n = {
                let mut map = std::collections::HashMap::new();
                if let Some(ref at) = media.alternative_titles {
                    if let Some(ref en) = at.en { if !en.is_empty() { map.insert("english".to_string(), en.clone()); } }
                    if let Some(ref ja) = at.ja { if !ja.is_empty() { map.insert("native".to_string(), ja.clone()); } }
                }
                map.insert("romaji".to_string(), media.title.clone());
                map
            };

            let genres: Vec<String> = media.genres.unwrap_or_default()
                .into_iter().map(|g| g.name).collect();

            let rating_str = media.rating.as_deref().unwrap_or("").to_lowercase();
            let is_nsfw = rating_str.contains("rx") || rating_str.contains("hentai")
                || media.nsfw.as_deref() == Some("black")
                || genres.iter().any(|g| { let gl = g.to_lowercase(); gl == "hentai" || gl == "erotica" });

            let staff: Vec<StaffMember> = media.authors.unwrap_or_default()
                .into_iter()
                .map(|a| {
                    let name = match (a.first_name.as_deref(), a.last_name.as_deref()) {
                        (Some(f), Some(l)) if !f.is_empty() && !l.is_empty() => format!("{} {}", f, l),
                        (Some(f), _) => f.to_string(),
                        (_, Some(l)) => l.to_string(),
                        _ => String::new(),
                    };
                    StaffMember { name, role: "Author".to_string(), image: None }
                })
                .filter(|s| !s.name.is_empty())
                .collect();

            let relations: Vec<TrackerRelation> = media.related_anime.unwrap_or_default()
                .into_iter()
                .map(|r| TrackerRelation {
                    relation_type: r.relation_type.to_uppercase(),
                    media: TrackerMedia {
                        tracker_id: format!("anime:{}", r.node.id),
                        tracker_url: None,
                        cross_ids: HashMap::new(),
                        content_type: ContentType::Anime,
                        title: r.node.title,
                        alt_titles: vec![],
                        title_i18n: Default::default(),
                        synopsis: None, cover_image: None, banner_image: None,
                        episode_count: None, chapter_count: None, status: None,
                        genres: vec![], tags: vec![], nsfw: false,
                        release_date: None, end_date: None, rating: None,
                        trailer_url: None, format: r.node.media_type,
                        studio: None, characters: vec![], staff: vec![], relations: vec![], episode_duration: None
                    },
                })
                .chain(
                    media.related_manga.unwrap_or_default()
                        .into_iter()
                        .map(|r| TrackerRelation {
                            relation_type: r.relation_type.to_uppercase(),
                            media: TrackerMedia {
                                tracker_id: format!("manga:{}", r.node.id),
                                tracker_url: None,
                                cross_ids: HashMap::new(),
                                content_type: ContentType::Manga,
                                title: r.node.title,
                                alt_titles: vec![],
                                title_i18n: Default::default(),
                                synopsis: None, cover_image: None, banner_image: None,
                                episode_count: None, chapter_count: None, status: None,
                                genres: vec![], tags: vec![], nsfw: false,
                                release_date: None, end_date: None, rating: None,
                                trailer_url: None, format: r.node.media_type,
                                studio: None, characters: vec![], staff: vec![], relations: vec![], episode_duration: None
                            },
                        })
                )
                .collect();

            let tracker_media = TrackerMedia {
                tracker_id: format!("manga:{}", media.id),
                tracker_url: Some(format!("https://myanimelist.net/manga/{}", media.id)),
                cross_ids: {
                    let mut m = HashMap::new();
                    m.insert("mal".to_string(), format!("manga:{}", media.id));
                    m
                },
                content_type: ContentType::Manga,
                title: media.title.clone(),
                alt_titles,
                title_i18n,
                synopsis: media.synopsis,
                cover_image: media.main_picture.as_ref().map(|p| p.large.clone().unwrap_or(p.medium.clone())),
                banner_image: None,
                episode_count: None,
                chapter_count: media.num_chapters,
                status: media.status,
                genres,
                tags: vec![],
                nsfw: is_nsfw,
                release_date: media.start_date,
                end_date: media.end_date,
                rating: media.mean,
                trailer_url: None,
                format: media.media_type,
                studio: None,
                characters: vec![],
                staff,
                relations,
                episode_duration: None,
            };

            entries.push(UserListEntry {
                tracker_media_id: format!("manga:{}", media.id),
                title: media.title,
                poster: media.main_picture.map(|p| p.large.unwrap_or(p.medium)),
                content_type: ContentType::Manga,
                format: tracker_media.format.clone(),
                status: Some(status.status),
                progress: status.num_chapters_read.unwrap_or(0),
                score: status.score.map(|s| s as f64),
                start_date: status.start_date,
                end_date: status.finish_date,
                repeat_count: status.num_times_rewatched.unwrap_or(0),
                notes: status.comments,
                is_private: false,
                total_episodes: None,
                total_chapters: media.num_chapters,
                media: Some(tracker_media),
            });
        }

        Ok(entries)
    }

    async fn update_entry(
        &self,
        access_token: &str,
        params: UpdateEntryParams,
    ) -> CoreResult<()> {
        let (media_type, id) = Self::parse_media_id(&params.media_id);
        let url = format!("{}/{}/{}/my_list_status", MAL_API_BASE_URL, media_type, id);

        let mut form: Vec<(&str, String)> = Vec::new();

        if let Some(st) = params.status {
            form.push(("status", st));
        }
        if let Some(prog) = params.progress {
            let key = if media_type == "manga" {
                "num_chapters_read"
            } else {
                "num_watched_episodes"
            };
            form.push((key, prog.to_string()));
        }
        if let Some(score) = params.score {
            form.push(("score", (score.round() as i32).to_string()));
        }
        if let Some(repeat) = params.repeat_count {
            let key = if media_type == "manga" {
                "num_times_reread"
            } else {
                "num_times_rewatched"
            };
            form.push((key, repeat.to_string()));
        }
        if let Some(notes) = params.notes {
            form.push(("comments", notes));
        }

        let res = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .form(&form)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(CoreError::Network(format!(
                "Error updating MAL: {}",
                res.status()
            )))
        }
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let (media_type, id) = Self::parse_media_id(media_id);
        let url = format!("{}/{}/{}/my_list_status", MAL_API_BASE_URL, media_type, id);

        let res = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        Ok(res.status().is_success() || res.status() == reqwest::StatusCode::NOT_FOUND)
    }

    fn to_core_metadata(&self, cid: &str, media: &TrackerMedia) -> Metadata {
        let now = Utc::now().timestamp();

        let count = match media.content_type {
            ContentType::Anime => media.episode_count.unwrap_or(0),
            _ => media.chapter_count.unwrap_or(0),
        };

        let status = media
            .status
            .as_deref()
            .map(Self::normalize_status);

        Metadata {
            id: None,
            cid: cid.to_string(),
            source_name: self.name().to_string(),
            source_id: Some(media.tracker_id.clone()),
            subtype: media.format.clone(),
            title: media.title.clone(),
            alt_titles: media.alt_titles.clone(),
            title_i18n: media.title_i18n.clone(),
            synopsis: media.synopsis.clone(),
            cover_image: media.cover_image.clone(),
            banner_image: media.banner_image.clone(),
            eps_or_chapters: EpisodeData::Count(count),
            status,
            genres: media.genres.clone(),
            release_date: media.release_date.clone(),
            end_date: media.end_date.clone(),
            rating: media.rating,
            trailer_url: media.trailer_url.clone(),
            characters: media.characters.clone(),
            studio: media.studio.clone(),
            staff: media.staff.clone(),
            external_ids: json!({}),
            episode_duration: media.episode_duration,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MalUserResponse {
    pub id:      i64,
    pub name:    Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalListResponse {
    data: Vec<MalListNodeWrapper>,
}

#[derive(Debug, Deserialize)]
struct MalListNodeWrapper {
    node: MalMediaNode,
    list_status: MalListStatus,
}

#[derive(Debug, Deserialize)]
struct MalMediaNode {
    id: i32,
    title: String,
    main_picture: Option<MalPicture>,
    num_episodes: Option<i32>,
    num_chapters: Option<i32>,
    num_volumes: Option<i32>,
    alternative_titles: Option<MalAlternativeTitles>,
    start_date: Option<String>,
    end_date: Option<String>,
    synopsis: Option<String>,
    mean: Option<f32>,
    genres: Option<Vec<MalGenre>>,
    media_type: Option<String>,
    status: Option<String>,
    rating: Option<String>,
    nsfw: Option<String>,
    studios: Option<Vec<MalStudio>>,
    related_anime: Option<Vec<MalRelatedEdge>>,
    related_manga: Option<Vec<MalRelatedEdge>>,
    source: Option<String>,
    authors: Option<Vec<MalAuthor>>,
}

#[derive(Debug, Deserialize)]
struct MalAuthor {
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalAlternativeTitles {
    synonyms: Option<Vec<String>>,
    en: Option<String>,
    ja: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalGenre {
    id: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MalStudio {
    id: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MalRelatedEdge {
    node: MalRelatedNode,
    relation_type: String,
    relation_type_formatted: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalRelatedNode {
    id: i32,
    title: String,
    media_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalListStatus {
    status: String,
    score: Option<i32>,
    num_episodes_watched: Option<i32>,
    num_times_rewatched: Option<i32>,
    num_chapters_read: Option<i32>,
    comments: Option<String>,
    start_date: Option<String>,
    finish_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalPicture {
    medium: String,
    large: Option<String>,
}


#[derive(Debug, Deserialize)]
struct JikanSearchResponse {
    data: Vec<JikanMedia>,
}

#[derive(Debug, Deserialize)]
struct JikanSingleResponse {
    data: JikanMedia,
}

#[derive(Debug, Deserialize)]
struct JikanMedia {
    mal_id: i32,
    url: String,
    images: JikanImages,
    title: String,
    title_english: Option<String>,
    title_japanese: Option<String>,
    title_synonyms: Option<Vec<String>>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    episodes: Option<i32>,
    chapters: Option<i32>,
    status: Option<String>,
    score: Option<f32>,
    rating: Option<String>,
    synopsis: Option<String>,
    genres: Option<Vec<JikanEntity>>,
    explicit_genres: Option<Vec<JikanEntity>>,
    studios: Option<Vec<JikanEntity>>,
    authors: Option<Vec<JikanEntity>>,
    trailer: Option<JikanTrailer>,
    aired: Option<JikanDateRange>,
    published: Option<JikanDateRange>,
    relations: Option<Vec<JikanRelation>>,
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanImages {
    jpg: JikanImageFormat,
}

#[derive(Debug, Deserialize)]
struct JikanImageFormat {
    large_image_url: Option<String>,
    image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanTrailer {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanDateRange {
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanRelation {
    relation: String,
    entry: Vec<JikanEntity>,
}

#[derive(Debug, Deserialize)]
struct JikanEntity {
    mal_id: i32,
    #[serde(rename = "type")]
    entity_type: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct JikanCharacterResponse {
    data: Vec<JikanCharacterEdge>,
}

#[derive(Debug, Deserialize)]
struct JikanCharacterEdge {
    character: JikanPersonEntity,
    role: String,
    voice_actors: Option<Vec<JikanVoiceActorEdge>>,
}

#[derive(Debug, Deserialize)]
struct JikanVoiceActorEdge {
    person: JikanPersonEntity,
    language: String,
}

#[derive(Debug, Deserialize)]
struct JikanStaffResponse {
    data: Vec<JikanStaffEdge>,
}

#[derive(Debug, Deserialize)]
struct JikanStaffEdge {
    person: JikanPersonEntity,
    positions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct JikanPersonEntity {
    name: String,
    images: Option<JikanPersonImages>,
}

#[derive(Debug, Deserialize)]
struct JikanPersonImages {
    jpg: Option<JikanImageFormat>,
}

impl JikanMedia {
    fn into_tracker_media(self, content_type: ContentType) -> TrackerMedia {
        let prefix = if content_type == ContentType::Manga {
            "manga"
        } else {
            "anime"
        };

        let mut alt_titles = Vec::new();
        if let Some(t) = self.title_english.clone() {
            alt_titles.push(t);
        }
        if let Some(t) = self.title_japanese.clone() {
            alt_titles.push(t);
        }
        if let Some(synonyms) = self.title_synonyms {
            alt_titles.extend(synonyms);
        }

        let mut title_i18n: HashMap<String, String> = HashMap::new();
        if let Some(s) = &self.title_japanese {
            if !s.is_empty() { title_i18n.insert("native".to_string(), s.clone()); }
        }

        if !self.title.is_empty() {
            title_i18n.insert("romaji".to_string(), self.title.clone());
        }
        if let Some(s) = &self.title_english {
            if !s.is_empty() { title_i18n.insert("english".to_string(), s.clone()); }
        }

        let (release_date, end_date) = if let Some(aired) = self.aired {
            (aired.from, aired.to)
        } else if let Some(published) = self.published {
            (published.from, published.to)
        } else {
            (None, None)
        };

        let studio = self
            .studios
            .and_then(|mut s| if s.is_empty() { None } else { Some(s.remove(0).name) });

        let mut all_genres = Vec::new();
        if let Some(g) = self.genres {
            all_genres.extend(g.into_iter().map(|e| e.name));
        }
        if let Some(eg) = self.explicit_genres {
            all_genres.extend(eg.into_iter().map(|e| e.name));
        }

        let rating_str = self.rating.as_deref().unwrap_or("").to_lowercase();
        let is_nsfw = rating_str.contains("rx")
            || rating_str.contains("hentai")
            || all_genres.iter().any(|g| {
            let gl = g.to_lowercase();
            gl == "hentai" || gl == "erotica"
        });

        let episode_duration = self.duration.as_deref().and_then(|d| {
            let d = d.to_lowercase();
            if d.contains("unknown") { return None; }
            let mut mins = 0i32;
            if let Some(h) = d.split("hr").next()
                .and_then(|s| s.trim().parse::<i32>().ok()) {
                mins += h * 60;
            }
            if let Some(m) = d.split("min").next()
                .and_then(|s| s.split_whitespace().last())
                .and_then(|s| s.parse::<i32>().ok()) {
                mins += m;
            }
            if mins > 0 { Some(mins) } else { None }
        });

        let mut relations = Vec::new();
        if let Some(jikan_rels) = self.relations {
            for rel in jikan_rels {
                for entry in rel.entry {
                    let c_type = if entry.entity_type.to_lowercase() == "manga" {
                        ContentType::Manga
                    } else {
                        ContentType::Anime
                    };
                    let rel_prefix = if c_type == ContentType::Manga {
                        "manga"
                    } else {
                        "anime"
                    };

                    let related_media = TrackerMedia {
                        tracker_id: format!("{}:{}", rel_prefix, entry.mal_id),
                        tracker_url: None,
                        cross_ids: HashMap::new(),
                        content_type: c_type,
                        title: entry.name,
                        alt_titles: vec![],
                        title_i18n: Default::default(),
                        synopsis: None,
                        cover_image: None,
                        banner_image: None,
                        episode_count: None,
                        chapter_count: None,
                        status: None,
                        genres: vec![],
                        tags: vec![],
                        nsfw: false,
                        release_date: None,
                        end_date: None,
                        rating: None,
                        trailer_url: None,
                        format: Some(entry.entity_type),
                        studio: None,
                        characters: vec![],
                        staff: vec![],
                        relations: vec![],
                        episode_duration: None,
                    };

                    relations.push(TrackerRelation {
                        relation_type: rel.relation.clone(),
                        media: related_media,
                    });
                }
            }
        }

        let mut staff = Vec::new();
        if content_type == ContentType::Manga {
            if let Some(authors) = self.authors {
                staff.extend(authors.into_iter().map(|a| StaffMember {
                    name: a.name,
                    role: "Author".to_string(),
                    image: None,
                }));
            }
        }

        TrackerMedia {
            tracker_id: format!("{}:{}", prefix, self.mal_id),
            tracker_url: Some(self.url),
            cross_ids: {
                let mut m = HashMap::new();
                m.insert("mal".to_string(), format!("{}:{}", prefix, self.mal_id));
                m
            },
            content_type,
            title: self.title,
            alt_titles,
            title_i18n,
            synopsis: self.synopsis,
            cover_image: self.images.jpg.large_image_url.or(self.images.jpg.image_url),
            banner_image: None,
            episode_count: self.episodes,
            chapter_count: self.chapters,
            status: self.status,
            genres: all_genres,
            tags: vec![],
            nsfw: is_nsfw,
            release_date,
            end_date,
            rating: self.score,
            trailer_url: self.trailer.and_then(|t| t.url),
            format: self.media_type,
            studio,
            characters: vec![],
            staff,
            relations,
            episode_duration
        }
    }
}