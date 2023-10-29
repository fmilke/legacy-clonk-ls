use dashmap::DashMap;
use std::sync::RwLock;
use std::env;
use tokio;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter::{Parser, Tree, InputEdit, Point};
use simplelog::WriteLogger;

// mod analyze;
mod highlighting;

#[derive(Debug, Default, Clone, Copy)]
pub struct TokenTypes {
    comment: u32,
    number: u32,
    string: u32,
    pragma_strict: u32,
    appendto: u32,
    id: u32,
    var_scope: u32,
    nil: u32,
    keyword: u32,
    parameter: u32,
    method: u32,
    function: u32,
    modifier: u32,
}

#[derive(Debug)]
struct Document {
    #[allow(dead_code)]
    url: Url,
    tree: Tree,
}

impl Document {
    fn new(url: Url, tree: Tree) -> Self {
        Document { url, tree }
    }
}

struct Backend {
    client: Client,
                
    token_types: RwLock<TokenTypes>,
    documents: DashMap<Url, Document>,
}

impl Backend {
    fn parse_semantic_tokens_capabilities(&self, _params: &InitializeParams) -> Option<(TokenTypes, SemanticTokensLegend)> {
        // if let Some(ref text_document) = params.capabilities.text_document {
            // let mut s = self.state.write().unwrap();

            // if let Some(ref semantic_tokens) = text_document.semantic_tokens {
            //     let tts: &Vec<SemanticTokenType> = &semantic_tokens.token_types;

            //     for (i, tt) in tts.iter().enumerate() {
            //         if tt == &SemanticTokenType::COMMENT {
            //             s.tokens.comment = i as u32;
            //         }
            //     }
            // }
        // }

        let legend = SemanticTokensLegend {
            token_types: vec![
                SemanticTokenType::COMMENT,
                SemanticTokenType::STRING,
                SemanticTokenType::NUMBER,
                SemanticTokenType::MACRO,
                SemanticTokenType::KEYWORD,
                SemanticTokenType::TYPE,
                SemanticTokenType::PARAMETER,
                SemanticTokenType::METHOD,
                SemanticTokenType::FUNCTION,
                SemanticTokenType::MODIFIER,
            ],
            token_modifiers: vec![],
        };

        let lut = TokenTypes {
            comment: 0,
            string: 1,
            number: 2,
            pragma_strict: 3,
            appendto: 3,
            id: 4,
            var_scope: 4,
            nil: 4,
            keyword: 4,
            parameter: 6,
            method: 7,
            function: 8,
            modifier: 9,
        };

        Some((lut, legend))
    }

    fn add_document(&self, uri: Url, contents: String) -> std::result::Result<(), String> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_c4script::language())
            .expect("Loading c4scrpt grammar");

        if let Some(tree) = parser.parse(contents, None) {
            let doc = Document::new(uri.clone(), tree);
            self.documents.insert(uri, doc);
            Ok(())
        } else {
            Err(String::from("Could not parse document"))
        }
    }

    fn change_document(&self, uri: Url, contents: String) -> std::result::Result<(), String> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_c4script::language())
            .expect("Loading c4scrpt grammar");

        if let Some(ref mut doc) = self.documents.get_mut(&uri) {
            let start_byte = doc.tree.root_node().start_byte();
            let old_end_byte = doc.tree.root_node().end_byte();
            let old_end_position = doc.tree.root_node().end_position();
            let new_end_row = contents.chars().filter(|c| c == &'\n').count();
            let new_end_column = contents.chars().rev().position(|c| c == '\n').unwrap_or(0);

            doc.tree.edit(&InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte: contents.len(),
                start_position: Point { row: 0, column: 0, },
                old_end_position,
                new_end_position: Point { row: new_end_row, column: new_end_column, },
            });

            if let Some(new_tree) = parser.parse(&contents, Some(&doc.tree)) {                
                doc.tree = new_tree;
                Ok(())
            } else {
                Err(String::from("Could not update parse tree"))
            }
        } else {
            Err(String::from("Could not get document"))
        }
    }

    fn drop_document(&self, uri: &Url) {
        self.documents.remove(uri);
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "initializing server...")
            .await;

        self.client
            .log_message(MessageType::INFO, "parsed capabilities...")
            .await;

        // TODO: compare with client capabilities

        let mut semantic_tokens_capabilities: Option<SemanticTokensServerCapabilities> = None;
        
        if let Some((lut, legend)) = self.parse_semantic_tokens_capabilities(&params) {
            semantic_tokens_capabilities = Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    legend,
                    ..SemanticTokensOptions::default()
                })
            );

            if let Ok(mut tt) = self.token_types.write() {
                *tt = lut;
            }
        }

        let text_document_sync_capabilities =
            TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                semantic_tokens_provider: semantic_tokens_capabilities,
                text_document_sync: Some(text_document_sync_capabilities),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Err(e) = self.add_document(params.text_document.uri, params.text_document.text) {
            self.client.log_message(MessageType::INFO, e).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Err(e) = self.change_document(
            params.text_document.uri,
            params.content_changes[0].text.clone(),
        ) {
            self.client.log_message(MessageType::INFO, e).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.drop_document(&params.text_document.uri);
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let doc = self.documents.get(&uri).unwrap();

        // Get as ref?
        let lut = match self.token_types.read() {
            Ok(lut) => *lut,
            _ => TokenTypes::default(),
        };

        let tokens = highlighting::collect_tokens(&doc.tree, lut);
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "shutting down...")
            .await;

        Ok(())
    }

    async fn completion(&self, p: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, format!("Got completion: {:?}", p))
            .await;

        Ok(None)
    }
}

async fn start_language_server() {

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        token_types: RwLock::new(TokenTypes::default()),
        client,
        documents: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

const MOCK: &str = include_str!("../03.c");

fn start_as_cli() {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_c4script::language())
        .expect("Loading c4scrpt grammar");

    let tree = parser.parse(MOCK, None).unwrap();

    println!("{:?}", tree);

    highlighting::collect_tokens(&tree, TokenTypes::default());
}

#[tokio::main]
async fn main() {

    if let Ok(file) = std::fs::File::create("/home/fmi/lsp-log") {
        let _ = WriteLogger::init(log::LevelFilter::Trace, simplelog::Config::default(), file);
    }

    let launch_with_lsp = env::args().find(|a| a == "--lsp").is_some();

    if launch_with_lsp {
        start_language_server().await;
    } else {
        start_as_cli();
    }
}
