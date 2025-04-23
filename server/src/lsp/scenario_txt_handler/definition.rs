use crate::lsp::shared::definition::{Definition, Defs, IniDefsProvider, ValueType};
use anyhow::Context;
use lazy_static::lazy_static;
use std::{collections::HashMap, str::FromStr};

pub struct ScenarioTxtDefs;

impl IniDefsProvider for ScenarioTxtDefs {
    fn get_def(section_name: &str, key: &str) -> Option<&'static Definition> {
        let map = &*DEFS;
        match map.get(section_name) {
            Some(inner_map) => inner_map.get(key),
            None => None,
        }
    }
}

lazy_static! {
    static ref DEFS: Defs<'static> = init_definitions2();
}

const UNPARSED_DEFS: &str = include_str!("./scenario_txt_defs.csv");
type DefProducer = fn(ctx: &mut DefProducerContext);

pub struct DefProducerContext {
    section_name: &'static str,
    key: &'static str,
    translation_key: &'static str,
    value_type: ValueType,
    map: Defs<'static>,
}

impl DefProducerContext {
    pub fn section_name(&self) -> &str {
        self.section_name
    }

    pub fn key(&self) -> &str {
        self.key
    }

    pub fn add_with_section(&mut self, section_name: &'static str) {
        self.add_def(
            section_name,
            self.key,
            self.value_type.clone(),
            self.translation_key,
        );
    }

    pub fn add(&mut self) {
        self.add_def(
            self.section_name,
            self.key,
            self.value_type.clone(),
            self.translation_key,
        );
    }

    fn add_def(
        &mut self,
        section_name: &'static str,
        key_name: &'static str,
        value_type: ValueType,
        translation_key: &'static str,
    ) {
        let def = Definition {
            description: translation_key,
            value_type: value_type.clone(),
        };

        match self.map.get_mut(section_name) {
            Some(sub_map) => {
                sub_map.insert(key_name, def);
            }
            None => {
                let mut sub_map = HashMap::new();
                sub_map.insert(key_name, def);
                self.map.insert(section_name, sub_map);
            }
        }
    }
}

impl Default for DefProducerContext {
    fn default() -> Self {
        DefProducerContext {
            section_name: "",
            key: "",
            translation_key: "",
            value_type: ValueType::Id,
            map: HashMap::new(),
        }
    }
}

pub fn parse_defs<'a>(s: &'static str, producer: DefProducer) -> Defs<'a> {
    let mut ctx = DefProducerContext::default();

    for line in s.lines() {
        let mut parts = line.split('|');
        let section_name = parts
            .next()
            .with_context(|| format!("Getting section name from {}", &line))
            .expect("Getting Scenario.txt section");

        let key_name = parts
            .next()
            .with_context(|| format!("Getting key from {}", &line))
            .expect("Getting Scenario.txt key");

        let value_type = parts
            .next()
            .map(|v| ValueType::from_str(v).unwrap())
            .with_context(|| format!("Getting value type from {}", &line))
            .expect("Getting Scenario.txt value type");

        let translation_key = parts
            .next()
            .with_context(|| format!("Getting translation key from {}", &line))
            .expect("Getting Scenario.txt translation key");

        ctx.section_name = section_name;
        ctx.key = key_name;
        ctx.translation_key = translation_key;
        ctx.value_type = value_type;

        producer(&mut ctx);
    }

    ctx.map
}

fn init_definitions2() -> Defs<'static> {
    parse_defs(UNPARSED_DEFS, |ctx| {
        if ctx.section_name == "Player" {
            ctx.add_with_section("Player1");
            ctx.add_with_section("Player2");
            ctx.add_with_section("Player3");
            ctx.add_with_section("Player4");
        } else {
            ctx.add();
        }
    })
}
