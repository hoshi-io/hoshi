use crate::content::models::ContentType;
use crate::content::types::DiscoverQuery;
use crate::home::types::*;

fn key(k: &str) -> Label {
    Label::Key { key: k.into() }
}

fn tracker(name: &str, ct: &ContentType, sort: &str, status: Option<&str>) -> DiscoverQuery {
    DiscoverQuery::Tracker {
        tracker: name.into(),
        content_type: ct.clone(),
        query: None,
        sort: Some(sort.into()),
        genre: None,
        format: None,
        status: status.map(Into::into),
        tags: vec![],
    }
}

fn anilist(ct: &ContentType, sort: &str, status: Option<&str>) -> DiscoverQuery {
    tracker("anilist", ct, sort, status)
}

fn mangabaka(ct: &ContentType, sort: &str, status: Option<&str>) -> DiscoverQuery {
    tracker("mangabaka", ct, sort, status)
}

fn section(prefix: &str, id: &str, title: &str, ct: &ContentType, layout: SectionLayout, source: SectionSource) -> HomeSection {
    HomeSection {
        id: format!("{prefix}.{id}"),
        title: key(title),
        layout,
        content_type: Some(ct.clone()),
        source,
        hidden: false,
    }
}

fn search(prefix: &str, id: &str, title: &str, ct: &ContentType, query: DiscoverQuery) -> HomeSection {
    section(prefix, id, title, ct, SectionLayout::Carousel, SectionSource::Search { query })
}

fn builtin(prefix: &str, id: &str, title: &str, ct: &ContentType, builtin: BuiltinSection) -> HomeSection {
    section(prefix, id, title, ct, SectionLayout::Carousel, SectionSource::Builtin { builtin })
}

fn with_layout(mut s: HomeSection, layout: SectionLayout) -> HomeSection {
    s.layout = layout;
    s
}

/// Personal rows first, then what's new, then discovery. The hero always comes from AniList trending.
fn anime_sections(ct: &ContentType) -> Vec<HomeSection> {
    let p = ct.as_str();
    vec![
        builtin(p, "continue", "home.continue.continue_watching", ct, BuiltinSection::ContinueWatching),
        builtin(p, "new_episodes", "home.presets.new_episodes", ct, BuiltinSection::NewEpisodes),
        search(p, "trending", "home.trending", ct, anilist(ct, "TRENDING_DESC", None)),
        builtin(p, "recently_aired", "home.recently_aired.title", ct, BuiltinSection::RecentlyAired),
        builtin(p, "sequels", "home.presets.sequels", ct, BuiltinSection::UnstartedSequels),
        search(p, "seasonal", "home.simulcast", ct, anilist(ct, "POPULARITY_DESC", Some("ongoing"))),
        with_layout(search(p, "top_rated", "home.critically_acclaimed", ct, anilist(ct, "SCORE_DESC", None)), SectionLayout::Ranked),
        search(p, "upcoming", "home.upcoming", ct, anilist(ct, "POPULARITY_DESC", Some("upcoming"))),
        builtin(p, "random", "home.presets.random_planning", ct, BuiltinSection::RandomPlanning),
        search(p, "popular", "home.popular", ct, anilist(ct, "POPULARITY_DESC", None)),
    ]
}

fn manga_sections(ct: &ContentType) -> Vec<HomeSection> {
    let p = ct.as_str();
    vec![
        builtin(p, "continue", "home.continue.continue_reading", ct, BuiltinSection::ContinueWatching),
        search(p, "trending", "home.trending", ct, anilist(ct, "TRENDING_DESC", None)),
        search(p, "latest", "home.presets.latest_updates", ct, mangabaka(ct, "latest", None)),
        builtin(p, "sequels", "home.presets.sequels", ct, BuiltinSection::UnstartedSequels),
        with_layout(search(p, "top_rated", "home.critically_acclaimed", ct, mangabaka(ct, "score", None)), SectionLayout::Ranked),
        search(p, "ongoing", "home.presets.ongoing", ct, mangabaka(ct, "popularity", Some("ongoing"))),
        builtin(p, "random", "home.presets.random_planning", ct, BuiltinSection::RandomPlanning),
        search(p, "popular", "home.popular", ct, mangabaka(ct, "popularity", None)),
        builtin(p, "extensions", "home.presets.extensions", ct, BuiltinSection::Extensions),
    ]
}

fn novel_sections(ct: &ContentType) -> Vec<HomeSection> {
    let p = ct.as_str();
    vec![
        builtin(p, "continue", "home.continue.continue_reading", ct, BuiltinSection::ContinueWatching),
        search(p, "trending", "home.trending", ct, anilist(ct, "TRENDING_DESC", None)),
        search(p, "latest", "home.presets.latest_updates", ct, mangabaka(ct, "latest", None)),
        with_layout(search(p, "top_rated", "home.critically_acclaimed", ct, mangabaka(ct, "score", None)), SectionLayout::Ranked),
        search(p, "ongoing", "home.presets.ongoing", ct, mangabaka(ct, "popularity", Some("ongoing"))),
        with_layout(search(p, "completed", "home.presets.completed_series", ct, mangabaka(ct, "popularity", Some("completed"))), SectionLayout::Strip),
        builtin(p, "random", "home.presets.random_planning", ct, BuiltinSection::RandomPlanning),
    ]
}

fn tab(ct: ContentType) -> HomeTab {
    let p = ct.as_str();
    let (title, icon, sections) = match ct {
        ContentType::Anime => ("home.anime", "tv", anime_sections(&ct)),
        ContentType::Manga => ("home.manga", "book", manga_sections(&ct)),
        ContentType::Novel => ("home.novel", "book-text", novel_sections(&ct)),
    };

    HomeTab {
        id: p.into(),
        title: key(title),
        icon: icon.into(),
        hero: Some(format!("{p}.trending")),
        sections,
    }
}

pub fn default_layout() -> HomeLayout {
    HomeLayout {
        tabs: vec![tab(ContentType::Anime), tab(ContentType::Manga), tab(ContentType::Novel)],
    }
}
