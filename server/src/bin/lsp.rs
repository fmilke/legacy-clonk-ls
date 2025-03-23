use dashmap::DashMap;
use legacy_clonk_ls::core::embedding::Embedding;
use legacy_clonk_ls::lsp::doc::{DocType, Document};
use legacy_clonk_ls::lsp::token_types::TokenTypes;
use std::fs::OpenOptions;
use std::sync::RwLock;
use tokio;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::info;
use tree_sitter::{InputEdit, Point};

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
    embedding: Embedding,
}

impl Backend {
    fn parse_semantic_tokens_capabilities(
        &self,
        _params: &InitializeParams,
    ) -> Option<(TokenTypes, SemanticTokensLegend)> {
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
            operator: 0, // TODO
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
        info!("add_document endpoint triggered. uri: {}", uri);

        let doc_type;
        match DocType::from_uri(&uri) {
            Ok(dt) => {
                doc_type = dt;
            }
            Err(e) => {
                let s = e.to_string();
                tracing::error!("could not get doctype from url: {}", &s);
                return Err(s);
            }
        }

        info!("detected doctype for document: {:?}", &doc_type);

        let mut parser = doc_type.get_parser().expect("Could not load language");

        if let Some(tree) = parser.parse(&contents, None) {
            let doc = Document::new(uri.clone(), tree, contents, doc_type);
            self.documents.insert(uri, doc);
            Ok(())
        } else {
            Err(String::from("Could not parse document"))
        }
    }

    fn change_document(&self, uri: Url, contents: String) -> std::result::Result<(), String> {
        if let Some(ref mut doc) = self.documents.get_mut(&uri) {
            tracing::info!(
                "Changed document {}, having doc type {:?}",
                &uri,
                &doc.doc_type
            );
            let mut parser = doc.doc_type.get_parser().expect("Could not load language");

            let start_byte = doc.tree.root_node().start_byte();
            let old_end_byte = doc.tree.root_node().end_byte();
            let old_end_position = doc.tree.root_node().end_position();
            let new_end_row = contents.chars().filter(|c| c == &'\n').count();
            let new_end_column = contents.chars().rev().position(|c| c == '\n').unwrap_or(0);

            doc.tree.edit(&InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte: contents.len(),
                start_position: Point { row: 0, column: 0 },
                old_end_position,
                new_end_position: Point {
                    row: new_end_row,
                    column: new_end_column,
                },
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
                }),
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    //trigger_characters: Some(vec![String::from("(")]),
                    ..Default::default()
                }),
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

        let handler = doc.doc_type.get_handler();
        let tokens = handler.collect_semantic_tokens(&doc.tree, lut, doc.source.as_ref());
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

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, "hover triggered...")
            .await;

        let uri = params.text_document_position_params.text_document.uri;
        match self.documents.get(&uri) {
            Some(doc) => {
                let handler = doc.doc_type.get_handler();
                if let Some(text) =
                    handler.get_hover_text(&doc, params.text_document_position_params.position)
                {
                    let markup = MarkupContent {
                        value: text,
                        kind: MarkupKind::Markdown,
                    };

                    let contents = HoverContents::Markup(markup);
                    let response = Hover {
                        contents,
                        range: None,
                    };

                    return Ok(Some(response));
                }
            }
            _ => {}
        }
        //match self.documents.get(&uri) {
        //    Some(doc) => {
        //        if let Some(query) =
        //            doc.get_item_at_pos(params.text_document_position_params.position)
        //        {
        //            if let Some(text) = self.embedding.query_signature(query) {
        //                let markup = MarkupContent {
        //                    value: text,
        //                    kind: MarkupKind::Markdown,
        //                };

        //                let contents = HoverContents::Markup(markup);
        //                let response = Hover {
        //                    contents,
        //                    range: None,
        //                };

        //                return Ok(Some(response));
        //            }
        //        }
        //    }
        //    _ => {}
        //}

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
        embedding: Embedding::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[tokio::main]
async fn main() {
    let options = OpenOptions::new()
        .append(true)
        .create(true)
        .open("/home/fmi/log")
        .unwrap();

    let subscriber = tracing_subscriber::fmt()
        .with_writer(options)
        .with_file(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    start_language_server().await;
}
