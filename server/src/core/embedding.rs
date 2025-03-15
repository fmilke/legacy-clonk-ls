use super::signatures::C4DataType;
use crate::lsp::doc::QueryableItem;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

pub struct FnHelpParam {
    pub name: String,
    pub data_type: C4DataType,
    // todo:
    // reference
}

pub struct FnHelp {
    pub params: Vec<FnHelpParam>,
    pub description: Option<String>,
}

impl FnHelp {
    pub fn to_help_text(&self, name: &String) -> String {
        let mut params = String::new();
        let mut seperate = false;
        for p in self.params.iter() {
            if seperate {
                params.push_str(", ");
            }

            // remove to_string()
            params.push_str(p.data_type.moniker());
            params.push(' ');
            params.push_str(p.name.as_str());
            seperate = true;
        }

        let d = match self.description {
            Some(ref d) => format!("\n\n{}", d),
            _ => String::new(),
        };

        format!("```c4script\nfunc {}({})```\n\n{}", name, params, d)
    }
}

#[derive(Deserialize)]
pub struct ConstantHelp {
    #[serde(rename(deserialize = "dataType"))]
    pub data_type: C4DataType,
    pub description: String,
    pub value: Option<String>,
}

#[derive(Deserialize)]
pub struct ConstantHelpDto {
    #[serde(flatten)]
    pub help: ConstantHelp,
    pub identifier: String,
}

impl ConstantHelp {
    pub fn to_help_text(&self, name: &String) -> String {
        format!("{}\n\n{}", name, self.description)
    }
}

const CATEGORIES: &str = include_str!("./c4d_categories.json");

pub struct Embedding {
    fn_help: HashMap<String, FnHelp>,
    cons_help: HashMap<String, ConstantHelp>,
}

impl Embedding {
    pub fn new() -> Self {
        let mut help = HashMap::<String, FnHelp>::new();

        help.insert(String::from("InitializePlayer"), FnHelp { 
            params: vec![FnHelpParam {
                name: String::from("iPlr"),
                data_type: C4DataType::Int,
            }],
            description: Some(String::from(r"After joining a new player the engine calls the function InitializePlayer in the scenario script for that player. This function is called after the basic player objects as defined in Scenario.txt have been placed, so a preliminary starting position has been selected and the player's crew and starting material and buildings are present.")),
        });

        help.insert(
            String::from("RemovePlayer"),
            FnHelp {
                params: vec![FnHelpParam {
                    name: String::from("iPlr"),
                    data_type: C4DataType::Int,
                }],
                description: None,
            },
        );

        let mut cons_help = HashMap::<String, ConstantHelp>::new();

        let cons_help_item = serde_json::from_str::<Vec<ConstantHelpDto>>(CATEGORIES)
            .expect("Could not parse predefined constant help");

        for i in cons_help_item {
            cons_help.insert(i.identifier, i.help);
        }

        Embedding {
            fn_help: help,
            cons_help,
        }
    }

    pub fn query_signature(&self, query: QueryableItem) -> Option<String> {
        match query {
            QueryableItem::Function(fn_name) => {
                info!("querying function definition: {}", fn_name);
                let r = self.fn_help.get(&fn_name).map(|x| x.to_help_text(&fn_name));
                info!("function definition found: {}", r.is_some());
                r
            }
            QueryableItem::Constant(cons_name) => {
                info!("querying constant: {}", cons_name);
                let r = self.cons_help.get(&cons_name).map(|x| x.to_help_text(&cons_name));
                info!("constnat found: {}", r.is_some());
                r
            }
            _ => None,
        }
    }
}
