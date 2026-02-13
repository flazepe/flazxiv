use crate::{
    mongodb::{BookmarkTag, bookmark_tags::BookmarkTags},
    pixiv::{PIXIV_BOOKMARKS_PER_PAGE, PixivBookmarkPageBodyWork},
    routes::bookmarks::PaginationSort,
};
use anyhow::Result;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    Collection,
    bson::{Document, doc},
    options::FindOptions,
};
use std::fmt::Display;

#[derive(Debug)]
pub struct Bookmarks {
    collection: Collection<PixivBookmarkPageBodyWork>,
    pub tags: BookmarkTags,
}

impl Bookmarks {
    pub fn new(collection: Collection<PixivBookmarkPageBodyWork>, tags_collection: Collection<BookmarkTag>) -> Self {
        let tags = BookmarkTags::new(tags_collection);
        Self { collection, tags }
    }

    pub async fn count<T: Into<Option<Document>>>(&self, filter: T) -> Result<u64> {
        Ok(self.collection.count_documents(filter.into().unwrap_or_default()).await?)
    }

    pub async fn get<T: Display>(&self, id: T) -> Result<Option<PixivBookmarkPageBodyWork>> {
        Ok(self.collection.find_one(doc! { "_id": id.to_string() }).await?)
    }

    pub async fn find<T: Into<Option<Document>>>(
        &self,
        filter: T,
        offset: u64,
        mut limit: i64,
        sort: PaginationSort,
    ) -> Result<Vec<PixivBookmarkPageBodyWork>> {
        if limit > PIXIV_BOOKMARKS_PER_PAGE {
            limit = PIXIV_BOOKMARKS_PER_PAGE;
        }

        let sort = doc! {
            "_syncDate": match sort {
                PaginationSort::Ascending => 1,
                PaginationSort::Descending => -1,
            },
        };

        let find_options = FindOptions::builder().limit(limit).sort(sort).skip(offset).build();
        Ok(self.collection.find(filter.into().unwrap_or_default()).with_options(find_options).await?.try_collect().await?)
    }

    pub async fn insert_many(&self, bookmarks: Vec<PixivBookmarkPageBodyWork>) -> Result<()> {
        for bookmark in &bookmarks {
            for tag in &bookmark.tags {
                self.tags.increment(tag).await?;
            }
        }

        // The bookmarks should be reversed since pixiv sorts them by newest to oldest
        // We want the opposite for an accurate bookmark sync date for the initial database population (because we are looping from the oldest page to the newest page during the init)
        // We do this because pixiv does not include bookmark addition date, but they do sort bookmarks by the order they were added
        let bookmarks = bookmarks
            .into_iter()
            .rev()
            .map(|mut bookmark| {
                // This is needed for sorting by added date in the local database
                bookmark.sync_date = Some(Utc::now().to_rfc3339());
                bookmark.tags = bookmark.tags.into_iter().map(|tag| tag.to_lowercase()).collect();
                bookmark
            })
            .collect::<Vec<PixivBookmarkPageBodyWork>>();

        self.collection.insert_many(bookmarks).await?;
        Ok(())
    }

    pub async fn delete<T: Display>(&self, id: T) -> Result<()> {
        let id = id.to_string();

        if let Some(bookmark) = self.get(&id).await? {
            for tag in bookmark.tags {
                let Some(bookmark_tag) = self.tags.get(&tag).await? else { continue };

                if bookmark_tag.total - 1 == 0 {
                    self.tags.delete(&bookmark_tag.id).await?;
                } else {
                    self.tags.decrement(&bookmark_tag.id).await?;
                }
            }
        }

        self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(())
    }
}
