use crate::{CONFIG, REQWEST, USER_AGENT};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::{VecSkipError, serde_as};
use std::fmt::Display;

// pixiv's hard limit is 100
pub const PIXIV_BOOKMARKS_PER_PAGE: i64 = 100;

#[derive(Serialize, Deserialize, Debug)]
pub struct PixivBookmarks {
    pub body: PixivBookmarkPageBody,
}

impl PixivBookmarks {
    pub async fn get_page<T: Display>(page: i64, tag: T) -> Result<Self> {
        let offset = (page - 1) * PIXIV_BOOKMARKS_PER_PAGE;

        let res = REQWEST
            .get(format!(
                "https://www.pixiv.net/ajax/user/{}/illusts/bookmarks?offset={offset}&limit={PIXIV_BOOKMARKS_PER_PAGE}&rest=show&tag={tag}",
                CONFIG.pixiv_user_id,
            ))
            .header("user-agent", USER_AGENT)
            .header("cookie", format!("PHPSESSID={}", CONFIG.pixiv_phpsessid))
            .send()
            .await?;

        Ok(res.json().await?)
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PixivBookmarkPageBody {
    // Had to skip objects that couldn't be deserialized because deleted artworks contain bad properties
    #[serde_as(as = "VecSkipError<_>")]
    pub works: Vec<PixivBookmarkPageBodyWork>,

    pub total: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PixivBookmarkPageBodyWork {
    #[serde(rename(serialize = "_id"), alias = "_id")]
    pub id: String,

    // This will be set after fetching the bookmarks
    #[serde(rename(serialize = "_syncDate"), alias = "_syncDate")]
    pub sync_date: Option<String>,

    pub title: String,
    pub illust_type: u64,
    pub x_restrict: u64,
    pub restrict: u64,
    pub sl: u64,
    pub url: String,
    pub description: String,
    pub tags: Vec<String>,
    pub user_id: String,
    pub user_name: String,
    pub width: u64,
    pub height: u64,
    pub page_count: u64,
    pub is_bookmarkable: bool,
    pub bookmark_data: Option<PixivBookmarkPageBodyWorkBookmarkData>,
    pub alt: String,
    pub title_caption_translation: PixivBookmarkPageBodyWorkTitleCaptionTranslation,
    pub create_date: String,
    pub update_date: String,
    pub is_masked: bool,
    pub ai_type: u64,
    pub visibility_scope: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PixivBookmarkPageBodyWorkBookmarkData {
    pub id: String,
    pub private: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PixivBookmarkPageBodyWorkTitleCaptionTranslation {
    pub work_title: Option<String>,
    pub work_caption: Option<String>,
}
