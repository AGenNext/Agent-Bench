//! Publish the logic-reference page to a live SurrealDB via the Rust SDK (WS).
//! Run: AGENTBENCH_DB_TOKEN=<jwt> cargo run --example publish_page --features server
//!
//! Endpoint/ns/db default to the SurrealDB Cloud instance; override with
//! AGENTBENCH_DB_URL / AGENTBENCH_DB_NS / AGENTBENCH_DB_DB.

use surrealdb::engine::any;

const PAGES_SCHEMA: &str = include_str!("../schema/pages.surql");
const CONTENT: &str = include_str!("../schema/pages/agent-bench-logic-reference.md");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("AGENTBENCH_DB_URL")
        .unwrap_or_else(|_| "wss://schemadb-06ehsj292ppah8kbsk9pmnjjbc.aws-aps1.surreal.cloud".into());
    let ns = std::env::var("AGENTBENCH_DB_NS").unwrap_or_else(|_| "main".into());
    let db_name = std::env::var("AGENTBENCH_DB_DB").unwrap_or_else(|_| "main".into());
    let token = std::env::var("AGENTBENCH_DB_TOKEN").expect("set AGENTBENCH_DB_TOKEN");

    eprintln!("connecting to {url} ...");
    let db = any::connect(url).await?;
    db.authenticate(token).await?;
    db.use_ns(&ns).use_db(&db_name).await?;

    // Define the page table, then upsert the page document.
    db.query(PAGES_SCHEMA).await?.check()?;
    db.query(
        "UPSERT type::thing('page', $slug) SET slug = $slug, title = $title, \
         format = 'markdown', content = $content RETURN AFTER",
    )
    .bind(("slug", "agent-bench-logic-reference"))
    .bind(("title", "Agent-Bench — Logic Reference"))
    .bind(("content", CONTENT))
    .await?
    .check()?;

    // Read it back.
    let title: Option<String> = db
        .query("SELECT VALUE title FROM page:`agent-bench-logic-reference`")
        .await?
        .take(0)?;
    println!("stored page title = {title:?}");
    println!("read via API: GET {{endpoint}}/key/page/agent-bench-logic-reference");
    Ok(())
}
