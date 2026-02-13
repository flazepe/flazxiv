mod bookmark_tags;
mod bookmarks;

use crate::CONFIG;
use anyhow::Result;
use bookmarks::Bookmarks;
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MongoDB {
    pub bookmarks: Bookmarks,
}

impl MongoDB {
    pub async fn new() -> Result<Self> {
        let database = Client::with_uri_str(&CONFIG.mongodb_uri.to_string()).await?.database("flazxiv");
        let bookmarks = Bookmarks::new(database.collection("bookmarks"), database.collection("bookmark-tags"));
        Ok(Self { bookmarks })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookmarkTag {
    #[serde(rename(serialize = "_id"), alias = "_id")]
    pub id: String,

    pub name: String,
    pub total: u64,
}
