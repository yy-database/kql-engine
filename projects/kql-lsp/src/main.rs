use tower_lsp::{Client, LspService, Server};
use kql_lsp::KqlLanguageServer;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| KqlLanguageServer::new(client)).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
