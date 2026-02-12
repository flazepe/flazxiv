mod config;
mod mongodb;
mod pixiv;
mod routes;
mod sync;

use anyhow::Result;
use axum::{Router, routing::get, serve};
use config::Config;
use mongodb::MongoDB;
use reqwest::Client;
use std::sync::{LazyLock, OnceLock};
use sync::{bookmark_tags::sync_bookmark_tags, bookmarks::sync_bookmarks};
use tokio::{main, net::TcpListener, spawn};
use tracing_subscriber::fmt;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| Config::load().expect("Could not load config"));
pub static MONGODB: OnceLock<MongoDB> = OnceLock::new();
pub static REQWEST: LazyLock<Client> = LazyLock::new(Client::new);
pub const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

#[main]
async fn main() -> Result<()> {
    fmt::init();

    MONGODB.set(MongoDB::new().await?).expect("Could not set MongoDB");
    spawn(sync_bookmark_tags());
    spawn(sync_bookmarks());

    let app = Router::new()
        .route("/api/bookmark-tags", get(routes::bookmark_tags::handler))
        .route("/api/bookmarks", get(routes::bookmarks::handler))
        .route("/api/bookmarks/{id}/validate", get(routes::bookmarks_validate::handler));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    serve(listener, app).await?;

    Ok(())
}
