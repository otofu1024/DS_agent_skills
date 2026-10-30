use ds_semantic_scholar_mcp::{SemanticScholarClient, SemanticScholarServer};
use rmcp::ServiceExt;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let server = SemanticScholarServer::new(SemanticScholarClient::from_env()?);
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(())
}
