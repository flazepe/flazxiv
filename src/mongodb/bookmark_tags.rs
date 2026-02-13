use crate::mongodb::BookmarkTag;
use anyhow::Result;
use futures::TryStreamExt;
use mongodb::{
    Collection,
    bson::{Regex, doc},
    options::FindOneAndUpdateOptions,
};
use regex_syntax::escape;
use std::fmt::Display;

#[derive(Debug)]
pub struct BookmarkTags {
    collection: Collection<BookmarkTag>,
}

impl BookmarkTags {
    pub fn new(collection: Collection<BookmarkTag>) -> Self {
        Self { collection }
    }

    pub async fn get<T: Display>(&self, id: T) -> Result<Option<BookmarkTag>> {
        let id = id.to_string().to_lowercase();
        Ok(self.collection.find_one(doc! { "_id": id }).await?)
    }

    pub async fn find<T: Display>(&self, query: T) -> Result<Vec<BookmarkTag>> {
        let query = query.to_string();

        let filter = if query.is_empty() {
            doc! {}
        } else {
            doc! {
                "_id": Regex {
                    pattern: escape(&query),
                    options: "i".into(),
                },
            }
        };

        Ok(self.collection.find(filter).sort(doc! { "total": -1 }).limit(50).await?.try_collect().await?)
    }

    pub async fn increment<T: Display>(&self, id: T) -> Result<()> {
        let options = FindOneAndUpdateOptions::builder().upsert(true).build();

        self.collection
            .find_one_and_update(
                doc! { "_id": id.to_string().to_lowercase() },
                doc! { "$set": { "name": id.to_string() }, "$inc": { "total": 1 } },
            )
            .with_options(options)
            .await?;

        Ok(())
    }

    pub async fn decrement<T: Display>(&self, id: T) -> Result<()> {
        let id = id.to_string().to_lowercase();
        self.collection.find_one_and_update(doc! { "_id": id }, doc! { "$inc": { "total": -1 }}).await?;
        Ok(())
    }

    pub async fn delete<T: Display>(&self, id: T) -> Result<()> {
        let id = id.to_string().to_lowercase();
        self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(())
    }
}
