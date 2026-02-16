use crate::{CONFIG, MONGODB, pixiv::PixivBookmarkPageBody, routes::Response};
use axum::{Json, extract::Query};
use mongodb::bson::{Document, doc};
use serde::Deserialize;

pub async fn handler(query: Query<Pagination>) -> Json<Response<PixivBookmarkPageBody>> {
    let mongodb = MONGODB.get().unwrap();
    let tags = query.tags.to_string().to_lowercase().split_whitespace().take(5).map(|tag| tag.to_string()).collect::<Vec<String>>();
    let mut filter = None;

    if !tags.is_empty() {
        let mut tag_lists = vec![];

        for tag in tags {
            let mut normalized_tags = vec![];

            if let Some(pixiv_tags) = CONFIG.bookmark_tag_mappings.get(&tag) {
                normalized_tags.extend(pixiv_tags.iter().map(|pixiv_tag| pixiv_tag.to_lowercase()));
            }

            for resolved_bookmark_tag in mongodb.bookmarks.tags.resolve_from_name_or_id(&tag).await.unwrap_or_default() {
                normalized_tags.push(resolved_bookmark_tag.id);
            }

            normalized_tags.push(tag);

            tag_lists.push(normalized_tags);
        }

        filter = Some(doc! { "$and": tag_lists.iter().map(|list| doc! { "tags": { "$in": list } }).collect::<Vec<Document>>() });
    }

    let count = match mongodb.bookmarks.count(filter.clone()).await {
        Ok(count) => count,
        Err(error) => return Json(Response::Error(format!("{error:?}"))),
    };

    let bookmarks = match mongodb.bookmarks.find(filter, query.offset, query.limit, query.0.sort).await {
        Ok(bookmarks) => bookmarks,
        Err(error) => return Json(Response::Error(format!("{error:?}"))),
    };

    Json(Response::Data(PixivBookmarkPageBody { works: bookmarks, total: count }))
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default)]
    tags: String,

    #[serde(default)]
    offset: u64,

    #[serde(default = "Pagination::default_limit")]
    limit: i64,

    #[serde(default = "Pagination::default_sort")]
    sort: PaginationSort,
}

impl Pagination {
    fn default_limit() -> i64 {
        30
    }

    fn default_sort() -> PaginationSort {
        PaginationSort::Descending
    }
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PaginationSort {
    Ascending,
    Descending,
}
