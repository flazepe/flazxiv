use crate::{MONGODB, pixiv::bookmark_tags::PixivBookmarkTag, routes::Response};
use axum::Json;
use tracing::error;

pub async fn handler() -> Json<Response<Vec<PixivBookmarkTag>>> {
    let mongodb = MONGODB.get().unwrap();

    match mongodb.bookmark_tags.find().await {
        Ok(bookmark_tags) => Json(Response::Data(bookmark_tags)),
        Err(error) => {
            error!("An error occurred while trying to get bookmark tags: {error:?}");
            Json(Response::Error(format!("{error:?}")))
        },
    }
}
