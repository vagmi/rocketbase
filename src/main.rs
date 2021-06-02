use anyhow::{Result, Context};

mod db;
mod router;
mod server;
mod settings;
mod app_state;

#[tokio::main]
async fn main() -> Result<()> {
    let router = router::build_router().await?;
    server::serve(router).await.context("Unable to serve")
}
