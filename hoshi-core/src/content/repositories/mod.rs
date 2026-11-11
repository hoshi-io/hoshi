pub mod extension;
pub mod cache;
pub mod relations;
pub mod unit;
pub mod content;
pub(crate) const IN_CHUNK: usize = 500;

pub(crate) fn placeholders(n: usize) -> String {
    vec!["?"; n].join(",")
}
pub(crate) fn unique<'a>(ids: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen = std::collections::HashSet::new();
    ids.into_iter().filter(|id| seen.insert(*id)).collect()
}