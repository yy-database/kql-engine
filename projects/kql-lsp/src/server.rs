use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use kql_parser::parser::Parser;
use kql_types::Span;

#[derive(Debug)]
pub struct KqlLanguageServer {
    client: Client,
}

impl KqlLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Convert KQL span to LSP position
    fn span_to_range(&self, text: &str, span: Span) -> Range {
        let start_line = text[..span.start].lines().count() as u32;
        let start_col = text[..span.start]
            .lines()
            .last()
            .map(|l| l.len() as u32)
            .unwrap_or(0);

        let end_line = text[..span.end].lines().count() as u32;
        let end_col = text[..span.end]
            .lines()
            .last()
            .map(|l| l.len() as u32)
            .unwrap_or(0);

        Range {
            start: Position { line: start_line, character: start_col },
            end: Position { line: end_line, character: end_col },
        }
    }

    /// Parse KQL code and return diagnostics
    fn parse_and_get_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        let mut parser = Parser::new(text);
        let result = parser.parse();

        match result {
            Ok(_) => vec![],
            Err(err) => {
                let span = match err.kind() {
                    kql_types::KqlErrorKind::LexicalError { span, .. } => *span,
                    kql_types::KqlErrorKind::ParseError { span, .. } => *span,
                    kql_types::KqlErrorKind::SemanticError { span, .. } => *span,
                    kql_types::KqlErrorKind::LintError { span, .. } => *span,
                    _ => Span { start: 0, end: 10 },
                };

                vec![Diagnostic {
                    range: self.span_to_range(text, span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("kql-lsp".to_string()),
                    message: err.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }]
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for KqlLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), " ".to_string(), "@".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("kql-lsp".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "KQL Language Server".to_string(),
                version: Some("0.0.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "KQL Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        self.client
            .log_message(MessageType::INFO, format!("Document opened: {}", uri))
            .await;
        
        let diagnostics = self.parse_and_get_diagnostics(&text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        
        // Get full text from changes
        if let Some(change) = params.content_changes.first() {
            let text = &change.text;
            let diagnostics = self.parse_and_get_diagnostics(text);
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.client
            .log_message(MessageType::INFO, format!("Document closed: {}", uri))
            .await;
        
        // Clear diagnostics
        self.client
            .publish_diagnostics(uri, vec![], None)
            .await;
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = vec![
            // Keywords
            CompletionItem {
                label: "struct".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define a struct".to_string()),
                insert_text: Some("struct ${1:Name} {\n    ${2:field}: ${3:Type}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "enum".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define an enum".to_string()),
                insert_text: Some("enum ${1:Name} {\n    ${2:Variant}\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define a let binding".to_string()),
                insert_text: Some("let ${1:name} = ${2:value}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "namespace".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define a namespace".to_string()),
                insert_text: Some("namespace ${1:name}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            // Types
            CompletionItem {
                label: "Key".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("Primary key type".to_string()),
                insert_text: Some("Key<${1:i32}>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "ForeignKey".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("Foreign key type".to_string()),
                insert_text: Some("ForeignKey<${1:Entity}>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "String".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("String primitive".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "i32".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("32-bit integer".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "i64".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("64-bit integer".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "f64".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("64-bit float".to_string()),
                ..Default::default()
            },
            // Annotations
            CompletionItem {
                label: "@auto_increment".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Auto increment annotation".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "@unique".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Unique constraint".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "@primary_key".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Primary key annotation".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "@relation".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Relation annotation".to_string()),
                insert_text: Some("@relation(name: \"${1:name}\", foreign_key: \"${2:fk}\")".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "@layout(json)".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("JSON storage layout".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "@table".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Custom table name".to_string()),
                insert_text: Some("@table(\"${1:name}\")".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "@schema".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Schema annotation".to_string()),
                insert_text: Some("@schema(\"${1:name}\")".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            // Query methods
            CompletionItem {
                label: ".filter".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Filter query".to_string()),
                insert_text: Some(".filter { ${1:$.field == value} }".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: ".map".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Map projection".to_string()),
                insert_text: Some(".map { ${1:$.{field1, field2}} }".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: ".sort".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Sort order".to_string()),
                insert_text: Some(".sort { ${1:$.field.desc()} }".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: ".count".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Count aggregation".to_string()),
                insert_text: Some(".count(${1:*})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: ".avg".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Average aggregation".to_string()),
                insert_text: Some(".avg(${1:field})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: ".sum".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Sum aggregation".to_string()),
                insert_text: Some(".sum(${1:field})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ];

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _uri = params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;
        
        // Basic hover info
        let contents = MarkupContent {
            kind: MarkupKind::Markdown,
            value: r#"**KQL Language Server**

KQL (Query with ADTs Language) - A declarative data modeling and query language.

**Example:**
```kql
struct User {
    @auto_increment
    id: Key<i32>,
    name: String,
    email: String?
}

let active_users = User.filter { $.status == "active" }
```

**Common Types:**
- `Key<T>` - Primary key
- `ForeignKey<T>` - Foreign key reference
- `T?` - Optional type
- `[T]` - List type

**Annotations:**
- `@auto_increment` - Auto incrementing field
- `@unique` - Unique constraint
- `@table("name")` - Custom table name
- `@schema("name")` - Schema qualifier
- `@layout(json)` - JSON storage
- `@relation` - Relationship definition
"#.to_string(),
        };

        Ok(Some(Hover {
            contents: HoverContents::Markup(contents),
            range: None,
        }))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        
        self.client
            .log_message(MessageType::INFO, format!("Formatting request for: {}", uri))
            .await;
        
        // Basic formatting - could be expanded with proper formatting logic
        Ok(None)
    }
}
