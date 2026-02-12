use crate::{MONGODB, REQWEST, routes::Response};
use axum::{Json, extract::Path};
use reqwest::StatusCode;
use tracing::{error, info};

pub async fn handler(bookmark_id: Path<u32>) -> Json<Response<bool>> {
    let mongodb = MONGODB.get().unwrap();
    let bookmark_id = *bookmark_id;

    match mongodb.bookmarks.get(bookmark_id).await {
        Ok(bookmark) => {
            if bookmark.is_none() {
                return Json(Response::Data(false));
            }
        },
        Err(error) => {
            error!("An error occurred while trying to get bookmark {bookmark_id}: {error:?}");
            return Json(Response::Error(format!("{error:?}")));
        },
    }

    match REQWEST.get(format!("https://pixiv.net/artworks/{bookmark_id}")).send().await {
        Ok(res) => {
            if res.status() == StatusCode::NOT_FOUND {
                info!("Bookmark {bookmark_id} exists in the local database but was not found on pixiv. Deleting...");
                _ = mongodb.bookmarks.delete(bookmark_id).await;
                return Json(Response::Data(false));
            }
        },
        Err(error) => {
            error!("An error occurred while trying to get bookmark {bookmark_id} from pixiv: {error:?}");
            return Json(Response::Error(format!("{error:?}")));
        },
    }

    Json(Response::Data(true))
}
