use std::collections::HashMap;

use lazy_static::lazy_static;

type TransMap = HashMap<String, String>;

pub struct Translation;

const RAW_TRANS_DE: &str = include_str!("./de-DE.json");

lazy_static! {
    static ref TRANS_DE: TransMap = init_lang();
}

impl Translation {
    pub fn get_translation(key: &str) -> Option<&String> {
        let v = (&*TRANS_DE).get(key);
        tracing::info!("Trying to get translation for {} got {}", key, v.unwrap_or(&String::from("NONE")));
        v
    }
}

fn init_lang() -> TransMap {
    let value = serde_json::from_str(RAW_TRANS_DE)
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
                tracing::info!("Adding translation for {}", &t_key);
                collection.insert(t_key, s);
            }
            _ => {}
        }

        parts.pop();
    }
}
