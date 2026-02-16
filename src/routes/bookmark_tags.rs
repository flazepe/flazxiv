use crate::{MONGODB, mongodb::BookmarkTag, routes::Response};
use axum::{Json, extract::Query};
use serde::Deserialize;
use tracing::error;

pub async fn handler(query: Query<TagQuery>) -> Json<Response<Vec<BookmarkTag>>> {
    let mongodb = MONGODB.get().unwrap();

    match mongodb.bookmarks.tags.find(&query.query).await {
        Ok(mut bookmark_tags) => {
            let total = mongodb.bookmarks.count(None).await.unwrap_or(0);
            bookmark_tags.insert(0, BookmarkTag { id: "すべて".into(), name: Some("all".into()), total });
            Json(Response::Data(bookmark_tags))
        },
        Err(error) => {
            error!("An error occurred while trying to get bookmark tags: {error:?}");
            Json(Response::Error(format!("{error:?}")))
        },
    }
}

#[derive(Deserialize)]
pub struct TagQuery {
    #[serde(default)]
    query: String,
}
