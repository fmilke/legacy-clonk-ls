use std::{collections::HashMap, str::FromStr, sync::RwLock};

use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug)]
enum SupportedLang {
    De,
    En,
}

impl Default for SupportedLang {
    fn default() -> Self {
        SupportedLang::En
    }
}

impl FromStr for SupportedLang {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "de" || s.starts_with("de-") {
            Ok(SupportedLang::De)
        } else if s == "en" || s.starts_with("en-") {
            Ok(SupportedLang::En)
        } else {
            Err("Language not supported")
        }
    }
}

type TransMap = HashMap<String, String>;

pub struct Translation;

const RAW_TRANS_DE: &str = include_str!("./de-DE.json");
const RAW_TRANS_EN: &str = include_str!("./en-EN.json");

lazy_static! {
    static ref TRANS_DE: TransMap = init_lang(RAW_TRANS_DE);
}

lazy_static! {
    static ref TRANS_EN: TransMap = init_lang(RAW_TRANS_EN);
}

static LOCALE: RwLock<SupportedLang> = RwLock::new(SupportedLang::En);

impl Translation {

    fn get_translation_map(lang: SupportedLang) -> &'static TransMap {
        match lang {
            SupportedLang::De => {
                &*TRANS_DE
            }
            SupportedLang::En => {
                &*TRANS_EN
            }
        }
    }

    fn get_current_locale() -> SupportedLang {
        match LOCALE.read() {
            Ok(ref v) => {
                (*v).clone()
            }
            Err(e) => {
                tracing::error!("Could not get locale: {}", e);
                SupportedLang::default()
            }
        }
    }

    pub fn get_translation(key: &str) -> Option<&String> {
        let lang = Self::get_current_locale();
        let v = Self::get_translation_map(lang).get(key);
        tracing::info!("Trying to get translation for {} got {}", key, v.unwrap_or(&String::from("NONE")));
        v
    }

    pub fn configure(lang_tag: impl AsRef<str>) {
        tracing::info!("Requested language: {}", lang_tag.as_ref());

        let resolved = SupportedLang::from_str(lang_tag.as_ref()).unwrap_or_default();
        tracing::info!("Resolved supported language: {}", lang_tag.as_ref());

        match LOCALE.write() {
            Ok(mut guard) => {
                *guard = resolved;
            }
            Err(e) => {
                tracing::error!("Could not set locale: {}", e);
            }
        }
    }
}

fn init_lang(s: &str) -> TransMap {
    let value = serde_json::from_str(s)
        .expect("Could not parse de-DE");

    let mut parts: Vec<String> = vec![];
    let mut collection = TransMap::new();

    match value {
        serde_json::Value::Object(map) => {
            init_lang_parts(map, &mut parts, &mut collection);
        }
        _ => {
            panic!("Expected json object in root of translation");
        }
    }

    collection
}

fn init_lang_parts(
    map: serde_json::Map<String, serde_json::Value>,
    parts: &mut Vec<String>,
    collection: &mut TransMap,
) {
    for (key, value) in map {
        parts.push(key);

        match value {
            serde_json::Value::Object(map) => {
                init_lang_parts(map, parts, collection);
            }
            serde_json::Value::String(s) => {
                let t_key = parts.join(".");
                collection.insert(t_key, s);
            }
            _ => {}
        }

        parts.pop();
    }
}
