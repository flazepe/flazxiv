use crate::{MONGODB, mongodb::BookmarkTag, routes::Response};
use axum::Json;
use tracing::error;

pub async fn handler() -> Json<Response<Vec<BookmarkTag>>> {
    let mongodb = MONGODB.get().unwrap();

    match mongodb.bookmarks.tags.find().await {
        Ok(mut bookmark_tags) => {
            let total = mongodb.bookmarks.count(None).await.unwrap_or(0);
            bookmark_tags.insert(0, BookmarkTag { id: "".into(), name: "All".into(), total });
            Json(Response::Data(bookmark_tags))
        },
        Err(error) => {
            error!("An error occurred while trying to get bookmark tags: {error:?}");
            Json(Response::Error(format!("{error:?}")))
        },
    }
}
