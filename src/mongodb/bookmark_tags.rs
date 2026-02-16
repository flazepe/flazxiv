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
        let mut filter = doc! {};

        let query = query.to_string().to_lowercase();

        if !query.is_empty() {
            let mut conditions = vec![];

            for tag in query.split_whitespace() {
                conditions.push(doc! { "_id": &tag });
                conditions.push(doc! { "name": &tag });

                let regex_pattern = Regex { pattern: escape(tag), options: "".into() };
                conditions.push(doc! { "_id": &regex_pattern });
                conditions.push(doc! { "name": &regex_pattern });
            }

            if !conditions.is_empty() {
                filter = doc! { "$or": conditions };
            }
        }

        Ok(self.collection.find(filter).sort(doc! { "total": -1 }).limit(50).await?.try_collect().await?)
    }

    pub async fn resolve_from_name_or_id<T: Display>(&self, name_or_id: T) -> Result<Vec<BookmarkTag>> {
        let name_or_id = name_or_id.to_string().to_string();
        let filter = doc! { "$or": [{ "_id": &name_or_id }, { "name": name_or_id }] };
        Ok(self.collection.find(filter).sort(doc! { "total": -1 }).limit(50).await?.try_collect().await?)
    }

    pub async fn increment<T: Display>(&self, id: T) -> Result<()> {
        let id = id.to_string().to_lowercase();
        let options = FindOneAndUpdateOptions::builder().upsert(true).build();
        self.collection.find_one_and_update(doc! { "_id": id }, doc! { "$inc": { "total": 1 } }).with_options(options).await?;
        Ok(())
    }

    pub async fn decrement<T: Display>(&self, id: T) -> Result<()> {
        let id = id.to_string().to_lowercase();
        self.collection.find_one_and_update(doc! { "_id": id }, doc! { "$inc": { "total": -1 }}).await?;
        Ok(())
    }

    pub async fn set_name<T: Display, U: Display>(&self, id: T, name: U) -> Result<()> {
        let id = id.to_string().to_lowercase();
        let name = name.to_string().to_lowercase();
        self.collection.update_one(doc! { "_id": id }, doc! { "$set": { "name": name } }).await?;
        Ok(())
    }

    pub async fn delete<T: Display>(&self, id: T) -> Result<()> {
        let id = id.to_string().to_lowercase();
        self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(())
    }
}
