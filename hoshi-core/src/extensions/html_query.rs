use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use tracing::warn;

pub(crate) fn execute_selector(document: &Html, selector_str: &str) -> Vec<Value> {
    let parts: Vec<&str> = selector_str.split(',').map(|s| s.trim()).collect();
    let mut results = vec![];

    for part in parts {
        if let Some(r) = handle_has_contains_selector(document, part) {
            results.extend(r);
        } else if part.contains(":contains(") {
            if let Some(r) = handle_contains_selector(document, part) {
                results.extend(r);
            }
        } else {
            let sanitized = sanitize_selector(part);
            let sel = Selector::parse(&sanitized);
            match sel {
                Ok(sel) => {
                    let els: Vec<_> = document.select(&sel).map(|el| element_to_json(el)).collect();
                    results.extend(els);
                }
                Err(e) => warn!("[SELECTOR] parse error: {:?}", e),
            }
        }
    }

    results
}

fn handle_has_contains_selector(document: &Html, selector: &str) -> Option<Vec<Value>> {
    let re = Regex::new(
        r"^(.*?):has\(\s*>\s*([a-zA-Z0-9_-]+(?:\.[a-zA-Z0-9_-]+)*):contains\(([^)]+)\)\s*\)(.*)?$"
    ).unwrap();
    let caps = re.captures(selector)?;

    let outer_sel_str = caps.get(1).map_or("*", |m| m.as_str()).trim();
    let inner_tag     = caps.get(2).map_or("*", |m| m.as_str()).trim();
    let contains_text = caps.get(3).map_or("", |m| m.as_str()).trim();
    let suffix        = caps.get(4).map_or("", |m| m.as_str()).trim();

    let outer_sel_str = if outer_sel_str.is_empty() { "*" } else { outer_sel_str };
    let outer_sel = Selector::parse(&sanitize_selector(outer_sel_str)).ok()?;
    let inner_sel = Selector::parse(&sanitize_selector(inner_tag)).ok()?;

    let mut results = vec![];

    for outer_el in document.select(&outer_sel) {
        let has_match = outer_el.children()
            .filter_map(|c| scraper::ElementRef::wrap(c))
            .any(|child| {
                inner_sel.matches(&child) && child.text().collect::<String>().contains(contains_text)
            });

        if !has_match {
            continue;
        }

        if suffix.is_empty() {
            results.push(element_to_json(outer_el));
        } else {
            let suffix = suffix.trim();
            let (combinator, child_sel_str) = if suffix.starts_with('>') {
                (">", suffix.trim_start_matches('>').trim())
            } else {
                ("", suffix)
            };

            let child_sel = Selector::parse(&sanitize_selector(child_sel_str)).ok()?;

            if combinator == ">" {
                for child in outer_el.children().filter_map(|c| scraper::ElementRef::wrap(c)) {
                    if child_sel.matches(&child) {
                        results.push(element_to_json(child));
                    }
                }
            } else {
                for desc in outer_el.select(&child_sel) {
                    results.push(element_to_json(desc));
                }
            }
        }
    }

    Some(results)
}

fn handle_contains_selector(document: &Html, selector: &str) -> Option<Vec<Value>> {
    let re = regex::Regex::new(r"^(.*?):contains\(([^)]+)\)\s*(\+\s*\S+)?$").unwrap();
    let caps = re.captures(selector)?;

    let base_sel = caps.get(1).map_or("*", |m| m.as_str()).trim();
    let contains_text = caps.get(2).map_or("", |m| m.as_str()).trim();
    let sibling = caps.get(3).map_or("", |m| m.as_str());

    let base_sel = if base_sel.is_empty() { "*" } else { base_sel };
    let sanitized = sanitize_selector(base_sel);
    let sel = Selector::parse(&sanitized).ok()?;

    let mut results = vec![];

    for el in document.select(&sel) {
        let text: String = el.text().collect();
        if text.contains(contains_text) {
            if sibling.is_empty() {
                results.push(element_to_json(el));
            } else {
                let sibling_tag = sibling.trim_start_matches('+').trim();
                let sanitized_sib = sanitize_selector(sibling_tag);
                let sib_sel = Selector::parse(&sanitized_sib);
                if let Ok(sib_sel) = sib_sel {
                    let mut next = el.next_sibling();
                    while let Some(n) = next {
                        if let Some(el) = Html::parse_fragment(
                            &scraper::ElementRef::wrap(n)
                                .map(|e| e.html())
                                .unwrap_or_default()
                        ).select(&sib_sel).next() {
                            results.push(element_to_json(el));
                            break;
                        }
                        next = n.next_sibling();
                    }
                }
            }
        }
    }

    Some(results)
}

fn element_to_json(el: scraper::ElementRef) -> Value {
    let attrs: HashMap<String, String> = el.value().attrs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let own_text: String = el.children()
        .filter_map(|child| child.value().as_text().map(|t| t.to_string()))
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    serde_json::json!({
        "text": el.text().collect::<Vec<_>>().join(""),
        "own_text": own_text,
        "html": el.inner_html(),
        "outer": el.html(),
        "attrs": attrs,
    })
}

fn sanitize_selector(selector: &str) -> String {
    let re = Regex::new(r":not\(\s*:has\([^)]*\)\s*\)").unwrap();
    let s = re.replace_all(selector, "").to_string();

    let re2 = Regex::new(r":has\([^)]*\)").unwrap();
    let s = re2.replace_all(&s, "").to_string();

    let re3 = Regex::new(r":not\(\s*\)").unwrap();
    let s = re3.replace_all(&s, "").to_string();

    let re4 = Regex::new(r"\s*[+~>]\s*$").unwrap();
    let re5 = Regex::new(r"\[([^\]]*):([^\]]*)\]").unwrap();
    let s = re5.replace_all(&s, |caps: &regex::Captures| {
        format!("[{}\\:{}]", &caps[1], &caps[2])
    }).to_string();

    re4.replace_all(&s, "").to_string()
}