pub mod bookmark_tags;
pub mod bookmarks;
pub mod bookmarks_validate;

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Response<T> {
    Data(T),
    Error(String),
}
