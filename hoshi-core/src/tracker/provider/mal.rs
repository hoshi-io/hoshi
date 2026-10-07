pub(crate) use super::{
    TokenData, TrackerAuthConfig, TrackerMedia, TrackerProvider, TrackerRelation, UpdateEntryParams,
    UserListEntry,
};
use crate::content::models::{ContentType, EpisodeData, Metadata, StaffMember, Status};
use crate::error::{CoreError, CoreResult};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};

/// Official MAL API: only used for things that need the user's OAuth token
const MAL_API_BASE_URL: &str = "https://api.myanimelist.net/v2";
/// Tenrai (Jikan v4-compatible mirror): used for all public, unauthenticated
const TENRAI_API_BASE_URL: &str = "https://api.tenrai.org/v1";
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
            "currently_airing" | "currently_publishing" | "publishing" => Status::Ongoing,
            "not_yet_aired" | "not_yet_published" => Status::Planned,
            _ => Status::Ongoing,
        }
    }

    fn join_author_name(first: Option<&str>, last: Option<&str>) -> String {
        match (first.filter(|s| !s.is_empty()), last.filter(|s| !s.is_empty())) {
            (Some(f), Some(l)) => format!("{} {}", f, l),
            (Some(f), None)    => f.to_string(),
            (None,    Some(l)) => l.to_string(),
            _                  => String::new(),
        }
    }

    fn relation_stub(id: i32, title: String, content_type: ContentType, format: Option<String>) -> TrackerMedia {
        let prefix = match content_type {
            ContentType::Manga | ContentType::Novel => "manga",
            ContentType::Anime                      => "anime",
        };
        TrackerMedia {
            tracker_id:       format!("{}:{}", prefix, id),
            tracker_url:      None,
            cross_ids:        HashMap::new(),
            content_type,
            title,
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
            format,
            studio:           None,
            characters:       vec![],
            staff:            vec![],
            relations:        vec![],
            episode_duration: None,
        }
    }

    fn build_relations(
        related_anime: Option<Vec<MalRelatedEdge>>,
        related_manga: Option<Vec<MalRelatedEdge>>,
    ) -> Vec<TrackerRelation> {
        let anime_rels = related_anime.unwrap_or_default().into_iter().map(|r| TrackerRelation {
            relation_type: r.relation_type.to_uppercase(),
            media: Self::relation_stub(r.node.id, r.node.title, ContentType::Anime, r.node.media_type),
        });
        let manga_rels = related_manga.unwrap_or_default().into_iter().map(|r| TrackerRelation {
            relation_type: r.relation_type.to_uppercase(),
            media: Self::relation_stub(r.node.id, r.node.title, ContentType::Manga, r.node.media_type),
        });
        anime_rels.chain(manga_rels).collect()
    }

    fn build_alt_titles(at: Option<&MalAlternativeTitles>) -> Vec<String> {
        let mut v = Vec::new();
        if let Some(at) = at {
            if let Some(en) = &at.en  { if !en.is_empty() { v.push(en.clone()); } }
            if let Some(ja) = &at.ja  { if !ja.is_empty() { v.push(ja.clone()); } }
            if let Some(sy) = &at.synonyms { v.extend(sy.clone()); }
        }
        v
    }

    fn build_title_i18n(title: &str, at: Option<&MalAlternativeTitles>) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("romaji".to_string(), title.to_string());
        if let Some(at) = at {
            if let Some(en) = &at.en { if !en.is_empty() { map.insert("english".to_string(), en.clone()); } }
            if let Some(ja) = &at.ja { if !ja.is_empty() { map.insert("native".to_string(),  ja.clone()); } }
        }
        map
    }

    fn is_nsfw(rating: Option<&str>, nsfw_flag: Option<&str>, genres: &[String]) -> bool {
        let rating_str = rating.unwrap_or("").to_lowercase();
        rating_str.contains("rx")
            || rating_str.contains("hentai")
            || nsfw_flag == Some("black")
            || genres.iter().any(|g| {
            let gl = g.to_lowercase();
            gl == "hentai" || gl == "erotica"
        })
    }

    /// Builds a TrackerMedia from a raw MAL media node. Shared by list-import
    /// (`mal_node_to_entry`) and single-entry lookup (`get_by_id`), since MAL's
    /// official API returns the same node shape in both places.
    fn mal_media_to_tracker_media(media: &MalMediaNode, content_type: ContentType) -> TrackerMedia {
        let alt_titles = Self::build_alt_titles(media.alternative_titles.as_ref());
        let title_i18n = Self::build_title_i18n(&media.title, media.alternative_titles.as_ref());

        let genres: Vec<String> = media.genres.clone().unwrap_or_default()
            .into_iter().map(|g| g.name).collect();

        let nsfw = Self::is_nsfw(
            media.rating.as_deref(),
            media.nsfw.as_deref(),
            &genres,
        );

        let relations = Self::build_relations(media.related_anime.clone(), media.related_manga.clone());

        let prefix = match content_type {
            ContentType::Manga | ContentType::Novel => "manga",
            ContentType::Anime                      => "anime",
        };
        let mal_url = format!("https://myanimelist.net/{}/{}", prefix, media.id);

        let staff: Vec<StaffMember> = if matches!(content_type, ContentType::Manga | ContentType::Novel) {
            media.authors.clone().unwrap_or_default()
                .into_iter()
                .filter_map(|a| {
                    let name = Self::join_author_name(a.first_name.as_deref(), a.last_name.as_deref());
                    if name.is_empty() { return None; }
                    Some(StaffMember { name, role: "Author".to_string(), image: None })
                })
                .collect()
        } else {
            vec![]
        };

        let cover = media.main_picture.as_ref()
            .map(|p| p.large.clone().unwrap_or_else(|| p.medium.clone()));

        let studio = media.studios.as_ref()
            .and_then(|studios| studios.first())
            .map(|s| s.name.clone());

        TrackerMedia {
            tracker_id:       format!("{}:{}", prefix, media.id),
            tracker_url:      Some(mal_url),
            cross_ids:        HashMap::from([("mal".to_string(), format!("{}:{}", prefix, media.id))]),
            content_type:     content_type.clone(),
            title:            media.title.clone(),
            alt_titles,
            title_i18n,
            synopsis:         media.synopsis.clone(),
            cover_image:      cover,
            banner_image:     None,
            episode_count:    media.num_episodes,
            chapter_count:    media.num_chapters,
            status:           media.status.clone(),
            genres,
            tags:             vec![],
            nsfw,
            release_date:     media.start_date.clone(),
            end_date:         media.end_date.clone(),
            rating:           media.mean,
            // MAL's official API doesn't expose a trailer URL; AniList fills this in the UI.
            trailer_url:      None,
            format:           media.media_type.clone(),
            studio,
            // MAL's official API has no characters endpoint; AniList fills this in the UI.
            characters:       vec![],
            staff,
            relations,
            episode_duration: None,
        }
    }


    fn snake(s: &str) -> String {
        s.trim().to_lowercase().replace(['-', ' '], "_")
    }

    /// Tenrai dates are ISO timestamps ("2006-04-03T00:00:00+00:00"); MAL's
    /// official API uses plain "YYYY-MM-DD", so trim to match.
    fn tenrai_date(s: Option<&str>) -> Option<String> {
        s.and_then(|s| s.get(..10)).map(|s| s.to_string())
    }

    /// Tenrai: "Finished Airing" -> "finished_airing", "Publishing" -> "currently_publishing".
    /// Keeps `TrackerMedia.status` in the same snake_case form the official API produced.
    fn tenrai_status(s: &str) -> String {
        match Self::snake(s).as_str() {
            "publishing" => "currently_publishing".to_string(),
            other => other.to_string(),
        }
    }

    /// Tenrai authors come as "Last, First"; flip to "First Last".
    fn tenrai_person_name(name: &str) -> String {
        match name.split_once(',') {
            Some((last, first)) => Self::join_author_name(Some(first.trim()), Some(last.trim())),
            None => name.trim().to_string(),
        }
    }

    fn tenrai_to_tracker_media(m: &TenraiMedia, content_type: ContentType) -> TrackerMedia {
        let is_manga = matches!(content_type, ContentType::Manga | ContentType::Novel);
        let prefix = if is_manga { "manga" } else { "anime" };

        let mut alt_titles: Vec<String> = Vec::new();
        for t in m.title_english.iter()
            .chain(m.title_japanese.iter())
            .chain(m.title_synonyms.iter())
        {
            if !t.is_empty() && !alt_titles.contains(t) {
                alt_titles.push(t.clone());
            }
        }

        let mut title_i18n = HashMap::new();
        title_i18n.insert("romaji".to_string(), m.title.clone());
        if let Some(en) = m.title_english.as_ref().filter(|s| !s.is_empty()) {
            title_i18n.insert("english".to_string(), en.clone());
        }
        if let Some(ja) = m.title_japanese.as_ref().filter(|s| !s.is_empty()) {
            title_i18n.insert("native".to_string(), ja.clone());
        }

        let genres: Vec<String> = m.genres.iter()
            .chain(m.explicit_genres.iter())
            .map(|g| g.name.clone())
            .collect();

        let nsfw = Self::is_nsfw(m.rating.as_deref(), None, &genres);

        let relations: Vec<TrackerRelation> = m.relations.iter()
            .flat_map(|r| {
                let rel_type = r.relation.trim().to_uppercase().replace(['-', ' '], "_");
                r.entry.iter().map(move |e| {
                    let ct = if e.kind == "manga" { ContentType::Manga } else { ContentType::Anime };
                    TrackerRelation {
                        relation_type: rel_type.clone(),
                        media: Self::relation_stub(e.mal_id, e.name.clone(), ct, None),
                    }
                })
            })
            .collect();

        let staff: Vec<StaffMember> = if is_manga {
            m.authors.iter()
                .filter_map(|a| {
                    let name = Self::tenrai_person_name(&a.name);
                    if name.is_empty() { return None; }
                    Some(StaffMember { name, role: "Author".to_string(), image: None })
                })
                .collect()
        } else {
            vec![]
        };

        let cover = m.images.as_ref()
            .and_then(|i| i.jpg.as_ref())
            .and_then(|j| j.large_image_url.clone().or_else(|| j.image_url.clone()));

        let dates = m.aired.as_ref().or(m.published.as_ref());

        TrackerMedia {
            tracker_id:       format!("{}:{}", prefix, m.mal_id),
            tracker_url:      Some(format!("https://myanimelist.net/{}/{}", prefix, m.mal_id)),
            cross_ids:        HashMap::from([("mal".to_string(), format!("{}:{}", prefix, m.mal_id))]),
            content_type,
            title:            m.title.clone(),
            alt_titles,
            title_i18n,
            synopsis:         m.synopsis.clone(),
            cover_image:      cover,
            banner_image:     None,
            episode_count:    m.episodes,
            chapter_count:    m.chapters,
            status:           m.status.as_deref().map(Self::tenrai_status),
            genres,
            tags:             vec![],
            nsfw,
            release_date:     dates.and_then(|d| Self::tenrai_date(d.from.as_deref())),
            end_date:         dates.and_then(|d| Self::tenrai_date(d.to.as_deref())),
            rating:           m.score,
            trailer_url:      m.trailer.as_ref().and_then(|t| t.url.clone()),
            format:           m.media_type.as_deref().map(Self::snake),
            studio:           m.studios.first().map(|s| s.name.clone()),
            // Tenrai has a characters endpoint, but AniList fills this in the UI.
            characters:       vec![],
            staff,
            relations,
            episode_duration: None,
        }
    }

    async fn tenrai_get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> CoreResult<Option<T>> {
        let res = self.client
            .get(format!("{}{}", TENRAI_API_BASE_URL, path))
            .query(query)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(CoreError::Network(format!("Tenrai returned {status}: {body}")));
        }

        let parsed = res.json::<T>().await
            .map_err(|e| CoreError::Parse(e.to_string()))?;
        Ok(Some(parsed))
    }

    async fn tenrai_media_list(
        &self,
        path: &str,
        query: &[(&str, String)],
        content_type: ContentType,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let list: Option<TenraiList> = self.tenrai_get(path, query).await?;
        let mut seen = HashSet::new();
        Ok(list
            .map(|l| l.data)
            .unwrap_or_default()
            .iter()
            .map(|m| Self::tenrai_to_tracker_media(m, content_type.clone()))
            .filter(|m| seen.insert(m.tracker_id.clone()))
            .collect())
    }

    /// Tenrai's `genres` filter takes numeric MAL genre ids. Accepts ids or
    /// names (comma separated) and resolves names via `/genres/{anime|manga}`.
    async fn resolve_genre_ids(&self, kind: &str, genre: &str) -> Option<String> {
        let wanted: Vec<&str> = genre.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        if wanted.is_empty() {
            return None;
        }

        let mut ids: Vec<String> = Vec::new();
        let mut table: Option<Vec<TenraiGenre>> = None;
        for g in wanted {
            if g.chars().all(|c| c.is_ascii_digit()) {
                ids.push(g.to_string());
                continue;
            }
            if table.is_none() {
                let res: Option<TenraiGenreList> = self
                    .tenrai_get(&format!("/genres/{}", kind), &[]).await.ok().flatten();
                table = Some(res.map(|r| r.data).unwrap_or_default());
            }
            if let Some(found) = table.as_ref()
                .and_then(|t| t.iter().find(|x| x.name.eq_ignore_ascii_case(g)))
            {
                ids.push(found.mal_id.to_string());
            }
        }

        if ids.is_empty() { None } else { Some(ids.join(",")) }
    }

    fn map_sort(sort: Option<&str>) -> Option<(&'static str, &'static str)> {
        match Self::snake(sort?).as_str() {
            "popularity" | "popular" | "trending" => Some(("popularity", "asc")),
            "score" | "rating" | "average_score" | "top" => Some(("score", "desc")),
            "title" | "name" => Some(("title", "asc")),
            "newest" | "latest" | "start_date" | "release_date" => Some(("start_date", "desc")),
            "members" => Some(("members", "desc")),
            "favorites" => Some(("favorites", "desc")),
            "rank" => Some(("rank", "asc")),
            _ => None,
        }
    }

    fn map_format(f: &str) -> String {
        let s = Self::snake(f);
        match s.as_str() {
            "light_novel" => "lightnovel".to_string(),
            "one_shot" => "oneshot".to_string(),
            _ => s,
        }
    }

    fn map_search_status(s: &str, is_manga: bool) -> Option<&'static str> {
        match Self::snake(s).as_str() {
            "finished" | "completed" | "complete" | "finished_airing" => Some("complete"),
            "releasing" | "airing" | "currently_airing" | "ongoing"
            | "publishing" | "currently_publishing" => {
                Some(if is_manga { "publishing" } else { "airing" })
            }
            "not_yet_released" | "not_yet_aired" | "not_yet_published"
            | "upcoming" | "planned" => Some("upcoming"),
            "hiatus" | "on_hiatus" if is_manga => Some("hiatus"),
            "cancelled" | "discontinued" if is_manga => Some("discontinued"),
            _ => None,
        }
    }

    fn mal_node_to_entry(
        node: MalListNodeWrapper,
        content_type: ContentType,
        score_format: Option<&str>,
    ) -> UserListEntry {
        let _ = score_format; // MAL scores are always POINT_10
        let media  = node.node;
        let status = node.list_status;

        let tracker_media = Self::mal_media_to_tracker_media(&media, content_type.clone());

        let (progress, repeat_count, total_episodes, total_chapters) =
            match content_type {
                ContentType::Anime => (
                    status.num_episodes_watched.unwrap_or(0),
                    status.num_times_rewatched.unwrap_or(0),
                    media.num_episodes,
                    None,
                ),
                _ => (
                    status.num_chapters_read.unwrap_or(0),
                    status.num_times_reread.unwrap_or(0),
                    None,
                    media.num_chapters,
                ),
            };

        UserListEntry {
            tracker_media_id: tracker_media.tracker_id.clone(),
            title:            media.title,
            poster:           media.main_picture.map(|p| p.large.unwrap_or(p.medium)),
            content_type,
            format:           tracker_media.format.clone(),
            status:           Some(status.status),
            progress,
            score:            status.score.map(|s| s as f64),
            start_date:       status.start_date,
            end_date:         status.finish_date,
            repeat_count,
            notes:            status.comments,
            is_private:       false,
            total_episodes,
            total_chapters,
            media:            Some(tracker_media),
        }
    }
}

#[async_trait]
impl TrackerProvider for MalProvider {
    fn name(&self) -> &'static str { "mal" }
    fn display_name(&self) -> &'static str { "MyAnimeList" }

    fn icon_url(&self) -> &'static str {
        "https://upload.wikimedia.org/wikipedia/commons/7/7a/MyAnimeList_Logo.png"
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Anime, ContentType::Manga]
    }

    fn auth_config(&self) -> TrackerAuthConfig {
        TrackerAuthConfig {
            oauth_flow: "pkce".to_string(),
            auth_url:   "https://myanimelist.net/v1/oauth2/authorize".to_string(),
            token_url:  Some("https://myanimelist.net/v1/oauth2/token".to_string()),
            client_id:  Some(MAL_CLIENT_ID.to_string()),
            scopes:     vec![],
        }
    }

    async fn validate_and_store_token(
        &self,
        access_token: &str,
        token_type: &str,
    ) -> CoreResult<TokenData> {
        let url = format!("{}/users/@me?fields=picture", MAL_API_BASE_URL);

        let res = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(CoreError::AuthError("error.tracker.invalid_credentials".into()));
        }

        let user_data: MalUserResponse = res.json().await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        Ok(TokenData {
            access_token:    access_token.to_string(),
            refresh_token:   None,
            token_type:      token_type.to_string(),
            expires_at:      Utc::now()
                .checked_add_signed(chrono::Duration::days(30))
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
            tracker_user_id: user_data.id.to_string(),
            display_name:    user_data.name,
            avatar_url:      user_data.picture,
            profile_url:     Some(format!("https://myanimelist.net/profile/{}", user_data.id)),
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
        nsfw: Option<bool>,
        status: Option<&str>,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let is_manga = matches!(content_type, ContentType::Manga | ContentType::Novel);
        let kind = if is_manga { "manga" } else { "anime" };

        let mut q: Vec<(&str, String)> = vec![
            ("limit", limit.clamp(1, 25).to_string()),
            ("page",  page.max(1).to_string()),
        ];

        let has_query = match query.map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) => { q.push(("q", s.to_string())); true }
            None    => false,
        };

        match Self::map_sort(sort) {
            Some((order_by, dir)) => {
                q.push(("order_by", order_by.to_string()));
                q.push(("sort", dir.to_string()));
            }
            // No explicit sort: keep Tenrai's relevance order for text queries,
            // otherwise fall back to "most popular" so browsing isn't random.
            None if !has_query => {
                q.push(("order_by", "popularity".to_string()));
                q.push(("sort", "asc".to_string()));
            }
            None => {}
        }

        let mut genre_ids = match genre {
            Some(g) => self.resolve_genre_ids(kind, g).await,
            None => None,
        };

        match format.map(str::trim).filter(|s| !s.is_empty()) {
            Some(f) => q.push(("type", Self::map_format(f))),
            None if matches!(content_type, ContentType::Novel) => {
                q.push(("type", "lightnovel".to_string()));
            }
            None => {}
        }

        if let Some(s) = status.and_then(|s| Self::map_search_status(s, is_manga)) {
            q.push(("status", s.to_string()));
        }

        if nsfw == Some(true) {
            // The frontend switch is "NSFW only", so actually restrict to adult
            // entries instead of just dropping the sfw filter. Anime has a
            // rating filter; manga doesn't, so use the Hentai genre (id 12).
            if is_manga {
                genre_ids = Some(match genre_ids {
                    Some(ids) => format!("{ids},12"),
                    None => "12".to_string(),
                });
            } else {
                q.push(("rating", "rx".to_string()));
            }
        } else {
            q.push(("sfw", "true".to_string()));
        }

        if let Some(ids) = genre_ids {
            q.push(("genres", ids));
        }

        self.tenrai_media_list(&format!("/{}", kind), &q, content_type).await
    }

    async fn get_by_id(&self, tracker_id: &str) -> CoreResult<Option<TrackerMedia>> {
        let (media_type, id) = Self::parse_media_id(tracker_id);

        let (kind, content_type) = if media_type == "manga" {
            ("manga", ContentType::Manga)
        } else {
            ("anime", ContentType::Anime)
        };

        let res: Option<TenraiSingle> = self
            .tenrai_get(&format!("/{}/{}/full", kind, id), &[])
            .await?;

        Ok(res.map(|r| Self::tenrai_to_tracker_media(&r.data, content_type)))
    }

    async fn get_home(&self) -> CoreResult<HashMap<String, Vec<TrackerMedia>>> {
        // Home is always served via AniList; MAL is list-import/manage + single-entry
        // lookup only, so there's nothing to fetch here.
        Ok(HashMap::new())
    }

    async fn get_user_list(
        &self,
        access_token: &str,
        _tracker_user_id: &str,
        score_format: Option<&str>,
    ) -> CoreResult<Vec<UserListEntry>> {
        let anime_fields = "list_status,node.id,node.title,node.main_picture,\
            node.alternative_titles,node.start_date,node.end_date,node.synopsis,\
            node.mean,node.nsfw,node.genres,node.media_type,node.status,\
            node.num_episodes,node.rating,node.studios,\
            node.related_anime,node.related_manga";
        let manga_fields = "list_status,node.id,node.title,node.main_picture,\
            node.alternative_titles,node.start_date,node.end_date,node.synopsis,\
            node.mean,node.nsfw,node.genres,node.media_type,node.status,\
            node.num_chapters,node.authors{first_name,last_name},\
            node.related_anime,node.related_manga";

        let anime_url = format!("{}/users/@me/animelist?fields={}&limit=1000", MAL_API_BASE_URL, anime_fields);
        let manga_url = format!("{}/users/@me/mangalist?fields={}&limit=1000", MAL_API_BASE_URL, manga_fields);

        let auth_header = format!("Bearer {}", access_token);

        let (anime_res, manga_res) = tokio::try_join!(
            self.client.get(&anime_url).header("Authorization", &auth_header).send(),
            self.client.get(&manga_url).header("Authorization", &auth_header).send(),
        ).map_err(|e| CoreError::Network(e.to_string()))?;

        let anime_list: MalListResponse = anime_res.json().await
            .map_err(|e| CoreError::Parse(e.to_string()))?;
        let manga_list: MalListResponse = manga_res.json().await
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let entries = anime_list.data.into_iter()
            .map(|n| Self::mal_node_to_entry(n, ContentType::Anime, score_format))
            .chain(
                manga_list.data.into_iter()
                    .map(|n| Self::mal_node_to_entry(n, ContentType::Manga, score_format))
            )
            .collect();

        Ok(entries)
    }

    async fn update_entry(&self, access_token: &str, params: UpdateEntryParams) -> CoreResult<()> {
        let (media_type, id) = Self::parse_media_id(&params.media_id);
        let url = format!("{}/{}/{}/my_list_status", MAL_API_BASE_URL, media_type, id);

        let mut form: Vec<(&str, String)> = Vec::new();

        if let Some(st)     = params.status       { form.push(("status", st)); }
        if let Some(prog)   = params.progress {
            let key = if media_type == "manga" { "num_chapters_read" } else { "num_watched_episodes" };
            form.push((key, prog.to_string()));
        }
        if let Some(score)  = params.score        { form.push(("score", (score.round() as i32).to_string())); }
        if let Some(repeat) = params.repeat_count {
            let key = if media_type == "manga" { "num_times_reread" } else { "num_times_rewatched" };
            form.push((key, repeat.to_string()));
        }
        if let Some(notes)  = params.notes        { form.push(("comments", notes)); }

        let res = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .form(&form)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(CoreError::Network(format!("error.tracker.update_failed: {}", res.status())))
        }
    }

    async fn delete_entry(&self, access_token: &str, media_id: &str) -> CoreResult<bool> {
        let (media_type, id) = Self::parse_media_id(media_id);
        let url = format!("{}/{}/{}/my_list_status", MAL_API_BASE_URL, media_type, id);

        let res = self.client
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
    node:        MalMediaNode,
    list_status: MalListStatus,
}

#[derive(Debug, Deserialize)]
struct MalMediaNode {
    id:                 i32,
    title:              String,
    main_picture:       Option<MalPicture>,
    num_episodes:       Option<i32>,
    num_chapters:       Option<i32>,
    alternative_titles: Option<MalAlternativeTitles>,
    start_date:         Option<String>,
    end_date:           Option<String>,
    synopsis:           Option<String>,
    mean:               Option<f32>,
    genres:             Option<Vec<MalGenre>>,
    media_type:         Option<String>,
    status:             Option<String>,
    rating:             Option<String>,
    nsfw:               Option<String>,
    studios:            Option<Vec<MalStudio>>,
    related_anime:      Option<Vec<MalRelatedEdge>>,
    related_manga:      Option<Vec<MalRelatedEdge>>,
    authors:            Option<Vec<MalAuthor>>,
}

#[derive(Debug, Deserialize, Clone)]
struct MalAuthor {
    first_name: Option<String>,
    last_name:  Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalAlternativeTitles {
    synonyms: Option<Vec<String>>,
    en:       Option<String>,
    ja:       Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct MalGenre {
    #[serde(rename = "id")]
    _id: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MalStudio {
    #[serde(rename = "id")]
    _id: i32,
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct MalRelatedEdge {
    node:                    MalRelatedNode,
    relation_type:           String,
    #[serde(rename = "relation_type_formatted")]
    _relation_type_formatted: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct MalRelatedNode {
    id:         i32,
    title:      String,
    media_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalListStatus {
    status:               String,
    score:                Option<i32>,
    num_episodes_watched: Option<i32>,
    num_times_rewatched:  Option<i32>,
    num_chapters_read:    Option<i32>,
    num_times_reread:     Option<i32>,
    comments:             Option<String>,
    start_date:           Option<String>,
    finish_date:          Option<String>,
}

#[derive(Debug, Deserialize)]
struct MalPicture {
    medium: String,
    large:  Option<String>,
}
// ───────────────────────── Tenrai (Jikan v4 shape) ─────────────────────────

#[derive(Debug, Deserialize)]
struct TenraiSingle {
    data: TenraiMedia,
}

#[derive(Debug, Deserialize)]
struct TenraiList {
    #[serde(default)]
    data: Vec<TenraiMedia>,
}

#[derive(Debug, Deserialize)]
struct TenraiGenreList {
    #[serde(default)]
    data: Vec<TenraiGenre>,
}

#[derive(Debug, Deserialize)]
struct TenraiGenre {
    mal_id: i32,
    name:   String,
}

/// Anime and manga share one struct; fields that don't apply are just absent.
#[derive(Debug, Deserialize)]
struct TenraiMedia {
    mal_id:         i32,
    title:          String,
    title_english:  Option<String>,
    title_japanese: Option<String>,
    #[serde(default)]
    title_synonyms: Vec<String>,
    #[serde(rename = "type")]
    media_type:     Option<String>,
    images:         Option<TenraiImages>,
    trailer:        Option<TenraiTrailer>,
    episodes:       Option<i32>,
    chapters:       Option<i32>,
    status:         Option<String>,
    aired:          Option<TenraiDates>,
    published:      Option<TenraiDates>,
    rating:         Option<String>,
    score:          Option<f32>,
    synopsis:       Option<String>,
    #[serde(default)]
    genres:         Vec<TenraiNamed>,
    #[serde(default)]
    explicit_genres: Vec<TenraiNamed>,
    #[serde(default)]
    studios:        Vec<TenraiNamed>,
    #[serde(default)]
    authors:        Vec<TenraiNamed>,
    #[serde(default)]
    relations:      Vec<TenraiRelation>,
}

#[derive(Debug, Deserialize)]
struct TenraiNamed {
    name: String,
}

#[derive(Debug, Deserialize)]
struct TenraiImages {
    jpg: Option<TenraiImageSet>,
}

#[derive(Debug, Deserialize)]
struct TenraiImageSet {
    image_url:       Option<String>,
    large_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TenraiTrailer {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TenraiDates {
    from: Option<String>,
    to:   Option<String>,
}

#[derive(Debug, Deserialize)]
struct TenraiRelation {
    relation: String,
    #[serde(default)]
    entry:    Vec<TenraiRelationEntry>,
}

#[derive(Debug, Deserialize)]
struct TenraiRelationEntry {
    mal_id: i32,
    #[serde(rename = "type", default)]
    kind:   String,
    name:   String,
}