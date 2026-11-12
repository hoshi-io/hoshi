use crate::content::models::ContentType;
use crate::content::types::DiscoverQuery;
use crate::home::types::*;

fn key(k: &str) -> Label {
    Label::Key { key: k.into() }
}

fn anilist(ct: &ContentType, sort: &str, status: Option<&str>, genre: Option<&str>) -> DiscoverQuery {
    DiscoverQuery::Tracker {
        tracker: "anilist".into(),
        content_type: ct.clone(),
        query: None,
        sort: Some(sort.into()),
        genre: genre.map(Into::into),
        format: None,
        status: status.map(Into::into),
        tags: vec![],
    }
}

fn search(prefix: &str, id: &str, title: &str, ct: &ContentType, query: DiscoverQuery) -> HomeSection {
    HomeSection {
        id: format!("{prefix}.{id}"),
        title: key(title),
        layout: SectionLayout::Carousel,
        content_type: Some(ct.clone()),
        source: SectionSource::Search { query },
        hidden: false,
    }
}

fn builtin(prefix: &str, id: &str, title: &str, ct: &ContentType, builtin: BuiltinSection) -> HomeSection {
    HomeSection {
        id: format!("{prefix}.{id}"),
        title: key(title),
        layout: SectionLayout::Carousel,
        content_type: Some(ct.clone()),
        source: SectionSource::Builtin { builtin },
        hidden: false,
    }
}

fn tab(ct: ContentType) -> HomeTab {
    let p = ct.as_str();
    let is_anime = ct == ContentType::Anime;
    let mut sections = vec![
        builtin(p, "continue", if is_anime { "home.continue.continue_watching" } else { "home.continue.continue_reading" }, &ct, BuiltinSection::ContinueWatching),
        search(p, "trending", "home.trending", &ct, anilist(&ct, "TRENDING_DESC", None, None)),
    ];
    if is_anime {
        sections.push(builtin(p, "recently_aired", "home.recently_aired.title", &ct, BuiltinSection::RecentlyAired));
    }
    sections.push(search(p, "popular", "home.popular", &ct, anilist(&ct, "POPULARITY_DESC", None, None)));
    if ct != ContentType::Novel {
        sections.push(search(p, "seasonal", if is_anime { "home.simulcast" } else { "home.latest" }, &ct,
                             anilist(&ct, "POPULARITY_DESC", Some("ongoing"), None)));
    }
    if is_anime {
        sections.push(search(p, "upcoming", "home.upcoming", &ct, anilist(&ct, "POPULARITY_DESC", Some("upcoming"), None)));
    }
    sections.extend([
        search(p, "recently_finished", "home.recently_finished", &ct, anilist(&ct, "END_DATE_DESC", Some("completed"), None)),
        search(p, "top_rated", "home.critically_acclaimed", &ct, anilist(&ct, "SCORE_DESC", None, None)),
    ]);
    if is_anime {
        sections.push(search(p, "action", "home.action", &ct, anilist(&ct, "SCORE_DESC", None, Some("Action"))));
    }

    let (title, icon) = match ct {
        ContentType::Anime => ("home.anime", "tv"),
        ContentType::Manga => ("home.manga", "book"),
        ContentType::Novel => ("home.novel", "book-text"),
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
