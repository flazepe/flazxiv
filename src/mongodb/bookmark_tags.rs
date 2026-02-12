use crate::pixiv::bookmark_tags::PixivBookmarkTag;
use anyhow::Result;
use futures::TryStreamExt;
use mongodb::{Collection, bson::doc, options::FindOptions};
use std::fmt::Display;

#[derive(Debug)]
pub struct BookmarkTags {
    collection: Collection<PixivBookmarkTag>,
}

impl BookmarkTags {
    pub fn new(collection: Collection<PixivBookmarkTag>) -> Self {
        Self { collection }
    }

    pub async fn get<T: Display>(&self, id: T) -> Result<Option<PixivBookmarkTag>> {
        Ok(self.collection.find_one(doc! { "_id": id.to_string() }).await?)
    }

    pub async fn find(&self) -> Result<Vec<PixivBookmarkTag>> {
        let find_options = FindOptions::builder().sort(doc! { "cnt": -1 }).build();
        Ok(self.collection.find(doc! {}).with_options(find_options).await?.try_collect().await?)
    }

    pub async fn delete_all(&self) -> Result<()> {
        self.collection.delete_many(doc! {}).await?;
        Ok(())
    }

    pub async fn insert_many(&self, tags: Vec<PixivBookmarkTag>) -> Result<()> {
        self.collection.insert_many(tags).await?;
        Ok(())
    }
}
