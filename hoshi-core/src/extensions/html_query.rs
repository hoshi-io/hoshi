use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::warn;

#[derive(Debug, Clone)]
enum Combinator {
    Descendant, // " "
    Child,      // ">"
    Adjacent,   // "+"
    Sibling,    // "~"
}

#[derive(Debug, Clone)]
struct SelectorStep {
    combinator: Combinator,
    base_selector: String,
    filters: Vec<PseudoFilter>,
}

#[derive(Debug, Clone)]
enum PseudoFilter {
    Contains(String),
    ContainsData(String),
    Has(String),
    Not(String),
}

pub(crate) fn execute_selector(document: &Html, selector_str: &str) -> Vec<Value> {
    let parts = split_selector_parts(selector_str);
    let mut results = vec![];

    for part in parts {
        let steps = parse_group(&part);
        let matched_elements = execute_steps(document, &steps);
        for el in matched_elements {
            results.push(element_to_json(el));
        }
    }

    results
}

fn split_selector_parts(selector: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for c in selector.chars() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            if depth > 0 { depth -= 1; }
            current.push(c);
        } else if c == ',' && depth == 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn parse_group(group: &str) -> Vec<SelectorStep> {
    let mut steps = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut chars = group.chars().peekable();
    let mut last_combinator = Combinator::Descendant;

    while let Some(c) = chars.next() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            if depth > 0 { depth -= 1; }
            current.push(c);
        } else if depth == 0 && (c == ' ' || c == '>' || c == '+' || c == '~') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                steps.push(parse_step(last_combinator.clone(), trimmed));
                current.clear();
            }

            let mut comb = c;
            if comb == ' ' {
                while let Some(&next_c) = chars.peek() {
                    if next_c == ' ' {
                        chars.next();
                    } else if next_c == '>' || next_c == '+' || next_c == '~' {
                        comb = chars.next().unwrap();
                        break;
                    } else {
                        break;
                    }
                }
            }

            last_combinator = match comb {
                '>' => Combinator::Child,
                '+' => Combinator::Adjacent,
                '~' => Combinator::Sibling,
                _ => Combinator::Descendant,
            };

            while let Some(&next_c) = chars.peek() {
                if next_c == ' ' {
                    chars.next();
                } else {
                    break;
                }
            }
        } else {
            current.push(c);
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        steps.push(parse_step(last_combinator, trimmed));
    }

    steps
}

fn parse_step(combinator: Combinator, step_str: &str) -> SelectorStep {
    let mut base_selector = String::new();
    let mut filters = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    let chars: Vec<char> = step_str.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ':' && depth == 0 {
            let remaining: String = chars[i..].iter().collect();
            if remaining.starts_with(":containsData(") {
                if !current.is_empty() {
                    if base_selector.is_empty() { base_selector = current.trim().to_string(); }
                    current.clear();
                }
                let (content, next_idx) = extract_parenthesized(&chars, i + 14);
                filters.push(PseudoFilter::ContainsData(content));
                i = next_idx;
                continue;
            } else if remaining.starts_with(":contains(") {
                if !current.is_empty() {
                    if base_selector.is_empty() { base_selector = current.trim().to_string(); }
                    current.clear();
                }
                let (content, next_idx) = extract_parenthesized(&chars, i + 10);
                filters.push(PseudoFilter::Contains(content));
                i = next_idx;
                continue;
            } else if remaining.starts_with(":has(") {
                if !current.is_empty() {
                    if base_selector.is_empty() { base_selector = current.trim().to_string(); }
                    current.clear();
                }
                let (content, next_idx) = extract_parenthesized(&chars, i + 5);
                filters.push(PseudoFilter::Has(content));
                i = next_idx;
                continue;
            } else if remaining.starts_with(":not(") {
                if !current.is_empty() {
                    if base_selector.is_empty() { base_selector = current.trim().to_string(); }
                    current.clear();
                }
                let (content, next_idx) = extract_parenthesized(&chars, i + 5);
                filters.push(PseudoFilter::Not(content));
                i = next_idx;
                continue;
            }
        }

        if c == '(' { depth += 1; }
        else if c == ')' { if depth > 0 { depth -= 1; } }

        current.push(c);
        i += 1;
    }

    if !current.is_empty() && base_selector.is_empty() {
        base_selector = current.trim().to_string();
    }
    if base_selector.is_empty() {
        base_selector = "*".to_string();
    }

    SelectorStep {
        combinator,
        base_selector,
        filters,
    }
}

fn extract_parenthesized(chars: &[char], start_idx: usize) -> (String, usize) {
    let mut content = String::new();
    let mut depth = 1;
    let mut i = start_idx;
    while i < chars.len() {
        let c = chars[i];
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 { break; }
        }
        content.push(c);
        i += 1;
    }
    (content, i + 1)
}

fn execute_steps<'a>(document: &'a Html, steps: &[SelectorStep]) -> Vec<scraper::ElementRef<'a>> {
    if steps.is_empty() { return Vec::new(); }

    let mut current_elements = Vec::new();
    let first_step = &steps[0];

    let sel = match Selector::parse(&first_step.base_selector) {
        Ok(s) => s,
        Err(e) => {
            warn!("[SELECTOR] parse error: {:?}", e);
            return Vec::new();
        }
    };

    for el in document.select(&sel) {
        if matches_filters(el, &first_step.filters) {
            current_elements.push(el);
        }
    }

    for step in &steps[1..] {
        let mut next_elements = Vec::new();
        let step_sel = match Selector::parse(&step.base_selector) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for el in &current_elements {
            match step.combinator {
                Combinator::Descendant => {
                    for desc in el.select(&step_sel) {
                        if matches_filters(desc, &step.filters) {
                            next_elements.push(desc);
                        }
                    }
                }
                Combinator::Child => {
                    for child in el.children().filter_map(scraper::ElementRef::wrap) {
                        if step_sel.matches(&child) && matches_filters(child, &step.filters) {
                            next_elements.push(child);
                        }
                    }
                }
                Combinator::Adjacent => {
                    let mut sib = el.next_sibling();
                    while let Some(n) = sib {
                        if let Some(sibling_el) = scraper::ElementRef::wrap(n) {
                            if step_sel.matches(&sibling_el) && matches_filters(sibling_el, &step.filters) {
                                next_elements.push(sibling_el);
                            }
                            break;
                        }
                        sib = n.next_sibling();
                    }
                }
                Combinator::Sibling => {
                    let mut sib = el.next_sibling();
                    while let Some(n) = sib {
                        if let Some(sibling_el) = scraper::ElementRef::wrap(n) {
                            if step_sel.matches(&sibling_el) && matches_filters(sibling_el, &step.filters) {
                                next_elements.push(sibling_el);
                            }
                        }
                        sib = n.next_sibling();
                    }
                }
            }
        }
        current_elements = deduplicate_elements(next_elements);
    }

    current_elements
}

fn deduplicate_elements<'a>(elements: Vec<scraper::ElementRef<'a>>) -> Vec<scraper::ElementRef<'a>> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for el in elements {
        if seen.insert(el.id()) {
            unique.push(el);
        }
    }
    unique
}

fn matches_filters(el: scraper::ElementRef, filters: &[PseudoFilter]) -> bool {
    for filter in filters {
        match filter {
            PseudoFilter::Contains(text) => {
                let el_text: String = el.text().collect();
                if !el_text.contains(text) { return false; }
            }
            PseudoFilter::ContainsData(text) => {
                let data: String = el.children()
                    .filter_map(|child| child.value().as_text().map(|t| t.to_string()))
                    .collect();
                if !data.contains(text) { return false; }
            }
            PseudoFilter::Has(selector_str) => {
                let nested_steps = parse_group(selector_str);
                if nested_steps.is_empty() || !evaluate_nested_steps(el, &nested_steps) {
                    return false;
                }
            }
            PseudoFilter::Not(selector_str) => {
                if let Ok(sel) = Selector::parse(selector_str) {
                    if sel.matches(&el) { return false; }
                }
            }
        }
    }
    true
}

fn evaluate_nested_steps(root: scraper::ElementRef, steps: &[SelectorStep]) -> bool {
    if steps.is_empty() { return false; }

    let first_step = &steps[0];
    let first_sel = match Selector::parse(&first_step.base_selector) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut current_elements = Vec::new();
    match first_step.combinator {
        Combinator::Child => {
            for child in root.children().filter_map(scraper::ElementRef::wrap) {
                if first_sel.matches(&child) && matches_filters(child, &first_step.filters) {
                    current_elements.push(child);
                }
            }
        }
        _ => {
            for desc in root.select(&first_sel) {
                if matches_filters(desc, &first_step.filters) {
                    current_elements.push(desc);
                }
            }
        }
    }

    if current_elements.is_empty() { return false; }

    for step in &steps[1..] {
        let mut next_elements = Vec::new();
        let step_sel = match Selector::parse(&step.base_selector) {
            Ok(s) => s,
            Err(_) => return false,
        };

        for el in &current_elements {
            match step.combinator {
                Combinator::Descendant => {
                    for desc in el.select(&step_sel) {
                        if matches_filters(desc, &step.filters) { next_elements.push(desc); }
                    }
                }
                Combinator::Child => {
                    for child in el.children().filter_map(scraper::ElementRef::wrap) {
                        if step_sel.matches(&child) && matches_filters(child, &step.filters) {
                            next_elements.push(child);
                        }
                    }
                }
                Combinator::Adjacent => {
                    let mut sib = el.next_sibling();
                    while let Some(n) = sib {
                        if let Some(sibling_el) = scraper::ElementRef::wrap(n) {
                            if step_sel.matches(&sibling_el) && matches_filters(sibling_el, &step.filters) {
                                next_elements.push(sibling_el);
                            }
                            break;
                        }
                        sib = n.next_sibling();
                    }
                }
                Combinator::Sibling => {
                    let mut sib = el.next_sibling();
                    while let Some(n) = sib {
                        if let Some(sibling_el) = scraper::ElementRef::wrap(n) {
                            if step_sel.matches(&sibling_el) && matches_filters(sibling_el, &step.filters) {
                                next_elements.push(sibling_el);
                            }
                        }
                        sib = n.next_sibling();
                    }
                }
            }
        }
        current_elements = deduplicate_elements(next_elements);
        if current_elements.is_empty() { return false; }
    }

    !current_elements.is_empty()
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
        "tag": el.value().name(),
    })
}