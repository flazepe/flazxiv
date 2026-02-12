mod bookmark_tags;
mod bookmarks;

use crate::CONFIG;
use anyhow::Result;
use bookmark_tags::BookmarkTags;
use bookmarks::Bookmarks;
use mongodb::Client;

#[derive(Debug)]
pub struct MongoDB {
    pub bookmark_tags: BookmarkTags,
    pub bookmarks: Bookmarks,
}

impl MongoDB {
    pub async fn new() -> Result<Self> {
        let database = Client::with_uri_str(&CONFIG.mongodb_uri.to_string()).await?.database("flazxiv");
        let bookmark_tags = BookmarkTags::new(database.collection("bookmark-tags"));
        let bookmarks = Bookmarks::new(database.collection("bookmarks"));
        Ok(Self { bookmark_tags, bookmarks })
    }
}
