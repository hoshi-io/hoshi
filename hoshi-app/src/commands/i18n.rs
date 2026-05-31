use serde_json::Value;

fn get_locale_str(lang: &str) -> Option<&'static str> {
    match lang {
        "en" => Some(include_str!("../../../locales/en.json")),
        "ar" => Some(include_str!("../../../locales/ar.json")),
        "de" => Some(include_str!("../../../locales/de.json")),
        "es" => Some(include_str!("../../../locales/es.json")),
        "fr" => Some(include_str!("../../../locales/fr.json")),
        "hi" => Some(include_str!("../../../locales/hi.json")),
        "id" => Some(include_str!("../../../locales/id.json")),
        "it" => Some(include_str!("../../../locales/it.json")),
        "ja" => Some(include_str!("../../../locales/ja.json")),
        "ko-KR" => Some(include_str!("../../../locales/ko-KR.json")),
        "ms" => Some(include_str!("../../../locales/ms.json")),
        "pl" => Some(include_str!("../../../locales/pl.json")),
        "pt-PT" => Some(include_str!("../../../locales/pt-PT.json")),
        "ru" => Some(include_str!("../../../locales/ru.json")),
        "th" => Some(include_str!("../../../locales/th.json")),
        "tl" => Some(include_str!("../../../locales/tl.json")),
        "tr" => Some(include_str!("../../../locales/tr.json")),
        "vi" => Some(include_str!("../../../locales/vi.json")),
        "zh-CN" => Some(include_str!("../../../locales/zh-CN.json")),
        "zh-TW" => Some(include_str!("../../../locales/zh-TW.json")),
        _ => None,
    }
}

#[tauri::command]
pub async fn load_locale(lang: String) -> Result<Value, String> {
    if !lang.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(format!("Invalid language code: {}", lang));
    }

    let content = get_locale_str(&lang)
        .ok_or_else(|| format!("Locale '{}' not found", lang))?;

    serde_json::from_str(content)
        .map_err(|e| format!("Invalid JSON in locale '{}': {}", lang, e))
}