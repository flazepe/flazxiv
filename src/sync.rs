use crate::{
    MONGODB,
    mongodb::MongoDB,
    pixiv::{PIXIV_BOOKMARKS_PER_PAGE, PixivBookmarks, PixivTags, PixivTagsBodyTagTranslationWrapper},
    routes::bookmarks::PaginationSort,
};
use anyhow::{Result, anyhow};
use kakasi::{IsJapanese, convert, is_japanese};
use std::{fmt::Display, thread::sleep, time::Duration};
use tracing::{error, info};

const SYNC_COOLDOWN_DURATION: Duration = Duration::from_secs(10);
const INSERT_ALL_COOLDOWN_DURATION: Duration = Duration::from_millis(500);

pub async fn sync_bookmarks() -> Result<()> {
    let mongodb = MONGODB.get().unwrap();
    let bookmark_count = mongodb.bookmarks.count(None).await.map_err(|error| anyhow!("Failed to get bookmark count: {error:?}"))?;

    if bookmark_count == 0 {
        info!("Local database has no bookmarks. Inserting all bookmarks...");
        insert_all_bookmarks(mongodb).await?;
        info!("Done inserting all bookmarks.");
    }

    info!("Syncing bookmarks...");

    loop {
        let mut page = 1;
        let mut next_page = true;
        let mut recent_pixiv_bookmark_ids = vec![];
        let mut new_bookmarks = vec![];

        while next_page {
            if page != 1 {
                info!("Checking page {page}... This may happen if bookmarks weren't synced in a while.");
            }

            let bookmarks = match PixivBookmarks::get_page(page, "").await {
                Ok(bookmarks) => bookmarks,
                Err(error) => {
                    error!("An error occurred while trying to get bookmark page {page}: {error:?}");
                    break;
                },
            };

            if page == 1 {
                recent_pixiv_bookmark_ids.extend(bookmarks.body.works.iter().map(|bookmark| bookmark.id.clone()));
            }

            // If somehow we're at the point where the current page is empty, let it start from the first page again (no need to break since it'll skip the loop anyway)
            if bookmarks.body.works.is_empty() {
                next_page = false;
            }

            for bookmark in &bookmarks.body.works {
                let bookmark_id = bookmark.id.clone();

                match mongodb.bookmarks.get(&bookmark_id).await {
                    Ok(existing_bookmark) => {
                        if existing_bookmark.is_some() {
                            // This page has an existing bookmark, so we won't bother inserting the current bookmark or looking through older pages
                            next_page = false;
                        } else {
                            new_bookmarks.push(bookmark.clone());
                        }
                    },
                    Err(error) => {
                        // Since it errored, let's just break the loop and let it check from the newest page again
                        error!("An error occurred while trying to get bookmark {bookmark_id}: {error:?}");
                        next_page = false;
                        break;
                    },
                }
            }

            if next_page {
                page += 1;
            }
        }

        if !new_bookmarks.is_empty() {
            let ids = new_bookmarks.iter().map(|bookmark| bookmark.id.clone()).collect::<Vec<String>>();

            if let Err(error) = mongodb.bookmarks.insert_many(new_bookmarks.clone()).await {
                error!("An error occurred while trying to insert bookmarks: {error:?}");
            } else {
                info!("{} new {} inserted: {}", ids.len(), if ids.len() == 1 { "bookmark" } else { "bookmarks" }, ids.join(", "));
            }
        }

        // Check for removed bookmarks by comparing the recent local bookmarks with pixiv's after everything is synced
        // This wouldn't be reliable if I removed some old bookmark that wasn't included in the list of recent ones, but whatever
        if !recent_pixiv_bookmark_ids.is_empty() {
            let recent_local_bookmarks =
                match mongodb.bookmarks.find(None, 0, recent_pixiv_bookmark_ids.len() as i64, PaginationSort::Descending).await {
                    Ok(recent_bookmarks) => recent_bookmarks,
                    Err(error) => {
                        error!("An error occurred while trying to get bookmarks: {error:?}");
                        continue;
                    },
                };

            let to_remove = recent_local_bookmarks.iter().filter(|bookmark| !recent_pixiv_bookmark_ids.contains(&bookmark.id));

            for bookmark in to_remove {
                if let Err(error) = mongodb.bookmarks.delete(&bookmark.id).await {
                    error!("An error occurred while trying to delete bookmark {}: {error:?}", bookmark.id);
                } else {
                    info!("Deleted bookmark {} because it was removed from recents.", bookmark.id);
                }
            }
        }

        sleep(SYNC_COOLDOWN_DURATION);
    }
}

pub async fn insert_all_bookmarks(mongodb: &MongoDB) -> Result<()> {
    let first_page = PixivBookmarks::get_page(1, "").await?;
    let total_pages = ((first_page.body.total as f64) / (PIXIV_BOOKMARKS_PER_PAGE as f64)).ceil() as i64;
    let mut page = total_pages;

    while page != 1 {
        info!("Inserting page {page}/{total_pages}...");

        let bookmarks = PixivBookmarks::get_page(page, "").await?;
        mongodb.bookmarks.insert_many(bookmarks.body.works).await?;

        page -= 1;
        sleep(INSERT_ALL_COOLDOWN_DURATION);
    }

    mongodb.bookmarks.insert_many(first_page.body.works).await?;

    Ok(())
}

pub async fn sync_bookmark_tag_translations<T: Display>(tags: Vec<T>) -> Result<()> {
    let mongodb = MONGODB.get().unwrap();

    for tag in tags {
        let id = tag.to_string().to_lowercase();

        let Some(bookmark_tag) = mongodb.bookmarks.tags.get(&id).await? else { continue };

        if bookmark_tag.name.is_some() {
            continue;
        }

        let pixiv_tags = match PixivTags::search(&id).await {
            Ok(pixiv_tags) => pixiv_tags,
            Err(error) => {
                error!(r#"An error occurred while trying to get pixiv tag "{id}": {error:?}"#);
                continue;
            },
        };

        // Add all related tags (which also sometimes include the translated version of the current tag)
        for pixiv_tag in &pixiv_tags.body.breadcrumbs.successor {
            let new_name = pixiv_tag.translation.en.split_whitespace().collect::<Vec<&str>>().join("_");
            mongodb.bookmarks.tags.set_name(&pixiv_tag.tag, new_name).await?;
        }

        // Add the romanized version of the current tag if it wasn't included in the breadcrumbs
        if !pixiv_tags.body.breadcrumbs.successor.iter().any(|entry| entry.tag.to_lowercase() == id) {
            let mut new_name = id.clone();

            if let PixivTagsBodyTagTranslationWrapper::HashMap(hashmap) = pixiv_tags.body.tag_translation {
                let romaji = hashmap.into_iter().find(|entry| entry.0.to_lowercase() == id).map(|entry| entry.1.romaji);

                if let Some(romaji) = romaji {
                    new_name = romaji;
                } else if is_japanese(&id) == IsJapanese::True {
                    new_name = convert(&id).romaji.split_whitespace().collect::<Vec<&str>>().join("_");
                }
            }

            mongodb.bookmarks.tags.set_name(&id, new_name).await?;
        }
    }

    Ok(())
}
