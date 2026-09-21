use uuid::Uuid;
use crate::state::AppState;
use crate::config::repository::ConfigRepository;

pub async fn show_adult(state: &AppState, user_id: i32) -> bool {
    ConfigRepository::get_config(&state.pool, user_id)
        .await
        .map(|c| c.general.show_adult_content)
        .unwrap_or(false)
}

pub fn generate_cid() -> String {
    Uuid::new_v4().to_string()
}

pub fn normalize_title(s: &str) -> String {
    s.to_lowercase()
        .replace([':', '-', '!', '?', '.', ',', '\'', '"', '·', '~'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let len1 = v1.len();
    let len2 = v2.len();
    let mut column: Vec<usize> = (0..=len1).collect();
    for x in 1..=len2 {
        column[0] = x;
        let mut last_diag = x - 1;
        for y in 1..=len1 {
            let old_diag = column[y];
            let cost = if v1[y - 1] == v2[x - 1] { 0 } else { 1 };
            column[y] = std::cmp::min(column[y] + 1, std::cmp::min(column[y - 1] + 1, last_diag + cost));
            last_diag = old_diag;
        }
    }
    column[len1]
}

pub fn similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 { return 1.0; }
    let max_len = std::cmp::max(s1.chars().count(), s2.chars().count());
    if max_len == 0 { return 1.0; }
    let dist = levenshtein_distance(s1, s2);
    1.0 - (dist as f64 / max_len as f64)
}

pub fn best_title_match<'a>(
    query_normalized: &str,
    titles: impl Iterator<Item = &'a str>,
) -> (f64, Option<&'a str>) {
    titles
        .filter(|t| !t.trim().is_empty())
        .map(|t| (similarity(query_normalized, &normalize_title(t)), t))
        .fold((0.0_f64, None), |acc, (score, t)| {
            if score > acc.0 { (score, Some(t)) } else { acc }
        })
}

pub fn apply_year_penalty(score: f64, query_year: Option<i64>, candidate_release_date: Option<&str>) -> f64 {
    if let (Some(qy), Some(release)) = (query_year, candidate_release_date) {
        if let Ok(dy) = release.chars().take(4).collect::<String>().parse::<i64>() {
            if (qy - dy).abs() > 1 {
                return score * 0.6;
            }
        }
    }
    score
}

#[macro_export]
macro_rules! diff_field {
    ($changes:expr, $prev_is_none:expr, $field:expr, $old:expr, $new:expr) => {{
        let old_s: Option<String> = $old;
        let new_s: String = $new;
        if $prev_is_none || old_s.as_deref() != Some(new_s.as_str()) {
            $changes.push(($field, old_s, new_s));
        }
    }};
}