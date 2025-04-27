use anyhow::Context;
use lazy_static::lazy_static;
use serde::Serialize;

const UNPARSED_DEFS: &str = include_str!("./ids");

lazy_static! {
    static ref DEFS: Vec<IdInfo> = init_id_infos();
}

fn init_id_infos() -> Vec<IdInfo> {
    let mut out = vec![];

    for line in UNPARSED_DEFS.lines() {
        let mut parts = line.split('|');
        let object_id = parts
            .next()
            .with_context(|| format!("Getting section name from {}", &line))
            .expect("Getting Scenario.txt section");

        let tlc = parts
            .next()
            .with_context(|| format!("Getting key from {}", &line))
            .expect("Getting Scenario.txt key");

        let german_name = parts
            .next()
            .with_context(|| format!("Getting key from {}", &line))
            .expect("Getting Scenario.txt key");

        let english_name = parts
            .next()
            .with_context(|| format!("Getting value type from {}", &line))
            .expect("Getting Scenario.txt value type");

        //let tlc = parts
        //    .next()
        //    .with_context(|| format!("Getting value type from {}", &line))
        //    .expect("Getting Scenario.txt value type");

        let mut name = String::from(german_name);
        name.push_str(" / ");
        name.push_str(english_name.as_ref());

        out.push(IdInfo {
            id: String::from(object_id),
            name,
            tlc: String::from(tlc),
        });
    }

    out
}

#[derive(Debug, Serialize)]
pub struct IdInfo {
    pub id: String,
    pub name: String,
    pub tlc: String,
}

pub struct Workspace;

impl Workspace {
    pub fn new() -> Self {
        Workspace {}
    }

    pub fn get_ids(&self) -> &'static Vec<IdInfo> {
        &DEFS
    }
}
