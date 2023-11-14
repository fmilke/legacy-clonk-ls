use dashmap::DashMap;
use highlighting::Highlighter;
use std::sync::RwLock;
use tokio;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter::{Parser, Tree, InputEdit, Point};

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
    parameter_type: u32,
    bool: u32,
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

struct OwnSemanticTokenType;
impl OwnSemanticTokenType {
    const PARAMETER_TYPE: SemanticTokenType = SemanticTokenType::new("parameterType");
    const ID: SemanticTokenType = SemanticTokenType::new("id");
    const BOOL: SemanticTokenType = SemanticTokenType::new("bool");
}

const NEGOTIATE_TOKENT_TYPES: bool = false;

struct Backend {
    client: Client,
                
    token_types: RwLock<TokenTypes>,
    documents: DashMap<Url, Document>,
}

impl Backend {
    fn parse_semantic_tokens_capabilities(&self, _params: &InitializeParams) -> Option<(TokenTypes, SemanticTokensLegend)> {
        // These are guaranteed by lsp
        let mut negotiated = vec![
            SemanticTokenType::COMMENT,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::MACRO,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::TYPE,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::METHOD,
        ];

        let mut lut = TokenTypes {
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
            parameter_type: 5,
            bool: 4,
        };

        // Set additional custom tokens
        if NEGOTIATE_TOKENT_TYPES {
            if let Some(ref doc) = _params.capabilities.text_document {
                if let Some(ref tt) = doc.semantic_tokens {
                    for t in &tt.token_types {
                        if t == &OwnSemanticTokenType::PARAMETER_TYPE {
                            lut.parameter_type = (negotiated.len()) as u32;
                            negotiated.push(OwnSemanticTokenType::PARAMETER_TYPE);
                        } else if t == &OwnSemanticTokenType::ID {
                            lut.id = (negotiated.len()) as u32;
                            negotiated.push(OwnSemanticTokenType::ID);
                        } else if t == &OwnSemanticTokenType::BOOL {
                            lut.bool = (negotiated.len()) as u32;
                            negotiated.push(OwnSemanticTokenType::BOOL);
                        }
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            lut.parameter_type = (negotiated.len()) as u32;
            negotiated.push(OwnSemanticTokenType::PARAMETER_TYPE);
            lut.id = (negotiated.len()) as u32;
            negotiated.push(OwnSemanticTokenType::ID);
            lut.bool = (negotiated.len()) as u32;
            negotiated.push(OwnSemanticTokenType::BOOL);
        }

        let legend = SemanticTokensLegend {
            token_types: negotiated,
            token_modifiers: vec![],
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

        let tokens = Highlighter::collect_tokens(&doc.tree, lut);
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

#[tokio::main]
async fn main() {

    // TODO: Properly configure logging
    /*
    if let Ok(file) = std::fs::File::create("/home/fmi/lsp-log") {
        let _ = WriteLogger::init(log::LevelFilter::Trace, simplelog::Config::default(), file);
    }
    */

    start_language_server()
        .await;
}
