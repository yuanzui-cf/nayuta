use crate::infra::error::{Result, user};
use rust_embed::Embed;
use std::collections::BTreeMap;

#[derive(Embed)]
#[folder = "assets/locale/"]
struct Assets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    ZhCn,
    ZhTw,
    ZhHk,
}

impl Locale {
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value
            .split(['.', '@'])
            .next()?
            .replace('_', "-")
            .to_lowercase();
        let parts: Vec<_> = normalized.split('-').collect();
        match parts.first().copied()? {
            "en" => Some(Self::En),
            "zh" if parts.contains(&"hk") || parts.contains(&"mo") => Some(Self::ZhHk),
            "zh" if parts.contains(&"tw") => Some(Self::ZhTw),
            "zh" if parts.contains(&"cn") || parts.contains(&"sg") => Some(Self::ZhCn),
            "zh" if parts.contains(&"hant") => Some(Self::ZhTw),
            "zh" if parts.contains(&"hans") => Some(Self::ZhCn),
            "c" | "posix" => Some(Self::En),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
            Self::ZhHk => "zh-HK",
        }
    }

    fn asset(self) -> &'static str {
        match self {
            Self::En => "en.json",
            Self::ZhCn => "zh_CN.json",
            Self::ZhTw => "zh_TW.json",
            Self::ZhHk => "zh_HK.json",
        }
    }
}

#[derive(Clone)]
pub struct I18n {
    pub locale: Locale,
    messages: BTreeMap<String, String>,
    english: BTreeMap<String, String>,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Locale::En)
    }
}

impl I18n {
    pub fn new(locale: Locale) -> Self {
        let load = |name| {
            serde_json::from_slice(&Assets::get(name).expect("embedded locale").data)
                .expect("valid locale JSON")
        };
        Self {
            locale,
            messages: load(locale.asset()),
            english: load("en.json"),
        }
    }

    pub fn detect(explicit: Option<&str>) -> Result<Self> {
        if let Some(value) = explicit {
            return Locale::parse(value)
                .map(Self::new)
                .ok_or_else(|| user("invalid_locale", value));
        }
        if let Ok(value) = std::env::var("NAYUTA_LOCALE")
            && let Some(locale) = Locale::parse(&value)
        {
            return Ok(Self::new(locale));
        }
        Ok(Self::new(
            system_candidates()
                .iter()
                .find_map(|s| Locale::parse(s))
                .unwrap_or(Locale::En),
        ))
    }

    pub fn t(&self, key: &str) -> String {
        self.messages
            .get(key)
            .or_else(|| self.english.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }

    pub fn error(&self, error: &crate::infra::error::Error) -> String {
        match error {
            crate::infra::error::Error::User { key, detail } => {
                format!("{}: {}", self.t(key), detail)
            }
            _ => format!("{}: {}", self.t("error"), error),
        }
    }
}

fn system_candidates() -> Vec<String> {
    let explicit = ["LC_ALL", "LC_MESSAGES"]
        .iter()
        .find_map(|key| std::env::var(key).ok().filter(|v| !v.is_empty()));
    if let Some(value) = explicit {
        return unix_candidates(&value);
    }
    #[cfg(target_os = "macos")]
    {
        let values: Vec<_> = sys_locale::get_locales().collect();
        if !values.is_empty() {
            return values;
        }
    }
    #[cfg(windows)]
    {
        return sys_locale::get_locales().collect();
    }
    #[cfg(not(windows))]
    {
        unix_candidates(&std::env::var("LANG").unwrap_or_default())
    }
}

fn unix_candidates(effective: &str) -> Vec<String> {
    let base = effective.split('.').next().unwrap_or_default();
    if base == "C" || base == "POSIX" {
        return vec!["en".into()];
    }
    let mut values: Vec<_> = std::env::var("LANGUAGE")
        .unwrap_or_default()
        .split(':')
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .collect();
    values.push(effective.to_owned());
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locales_use_region_before_script() {
        for (input, expected) in [
            ("zh_HK.UTF-8", Locale::ZhHk),
            ("zh-Hant-HK", Locale::ZhHk),
            ("zh-Hant-TW", Locale::ZhTw),
            ("zh-Hans", Locale::ZhCn),
            ("en_US.UTF-8", Locale::En),
            ("C.UTF-8", Locale::En),
        ] {
            assert_eq!(Locale::parse(input), Some(expected));
        }
        assert_eq!(Locale::parse("ja-JP"), None);
        assert_eq!(Locale::parse("zh"), None);
    }

    #[test]
    fn embedded_catalogs_are_complete_and_regional() {
        let english = I18n::default();
        for locale in [Locale::ZhCn, Locale::ZhTw, Locale::ZhHk] {
            let translated = I18n::new(locale);
            assert_eq!(
                english.english.keys().collect::<Vec<_>>(),
                translated.messages.keys().collect::<Vec<_>>()
            );
        }
        assert!(I18n::new(Locale::ZhTw).t("create").contains("部落格"));
        assert!(I18n::new(Locale::ZhHk).t("create").contains("網誌"));
    }
}
