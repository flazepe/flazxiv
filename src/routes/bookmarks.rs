use crate::{CONFIG, MONGODB, pixiv::bookmarks::PixivBookmarkPageBody, routes::Response};
use axum::{Json, extract::Query};
use mongodb::bson::{Document, Regex, doc};
use regex_syntax::escape;
use serde::Deserialize;

pub async fn handler(query: Query<Pagination>) -> Json<Response<PixivBookmarkPageBody>> {
    let tags = query.tags.to_string().to_lowercase().split_whitespace().take(5).map(|tag| tag.to_string()).collect::<Vec<String>>();

    let mut filter = None;

    if !tags.is_empty() {
        let mut regex_lists = vec![];

        for tag in tags {
            if let Some(pixiv_tags) = CONFIG.bookmark_tag_mappings.get(&tag) {
                regex_lists.push(
                    pixiv_tags
                        .iter()
                        .map(|pixiv_tag| Regex { pattern: format!("^{}$", escape(pixiv_tag)), options: "i".into() })
                        .collect::<Vec<Regex>>(),
                );
            } else {
                let is_exact_match = tag.starts_with('^') && tag.ends_with('$');

                regex_lists.push(vec![Regex {
                    pattern: if is_exact_match { format!("^{}$", escape(tag.trim_matches(['^', '$']))) } else { escape(&tag) },
                    options: "i".into(),
                }]);
            }
        }

        filter = Some(doc! { "$and": regex_lists.iter().map(|regexes| doc! { "tags": { "$in": regexes } }).collect::<Vec<Document>>() });
    }

    let mongodb = MONGODB.get().unwrap();

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
