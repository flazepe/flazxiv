use crate::{
    MONGODB,
    mongodb::MongoDB,
    pixiv::bookmarks::{PIXIV_BOOKMARKS_PER_PAGE, PixivBookmarks},
    routes::bookmarks::PaginationSort,
};
use anyhow::{Result, anyhow};
use std::{thread::sleep, time::Duration};
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

        while next_page {
            let bookmarks = match PixivBookmarks::get_page(page, "").await {
                Ok(bookmarks) => bookmarks,
                Err(error) => {
                    error!("An error occurred while trying to get bookmark page {page}: {error:?}");
                    break;
                },
            };

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
                        } else if let Err(error) = mongodb.bookmarks.insert_one(bookmark.clone()).await {
                            error!("An error occurred while trying to insert bookmark {bookmark_id}: {error:?}");
                        } else {
                            info!("New bookmark inserted: {bookmark_id}");
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

            // Check for removed bookmarks by comparing the recent local bookmarks with pixiv's after everything is synced
            // This wouldn't be reliable if I removed some old bookmark that wasn't included in the list of recent ones, but whatever
            // We only do this check if the page we grabbed was the latest one and we aren't checking an older page
            if page == 1 && !next_page {
                let recent_bookmarks = match mongodb.bookmarks.find(None, 0, PIXIV_BOOKMARKS_PER_PAGE, PaginationSort::Descending).await {
                    Ok(recent_bookmarks) => recent_bookmarks,
                    Err(error) => {
                        error!("An error occurred while trying to get bookmarks: {error:?}");
                        continue;
                    },
                };

                let removed = recent_bookmarks.iter().filter(|bookmark| !bookmarks.body.works.iter().any(|entry| entry.id == bookmark.id));

                for bookmark in removed {
                    if let Err(error) = mongodb.bookmarks.delete(&bookmark.id).await {
                        error!("An error occurred while trying to delete bookmark {}: {error:?}", bookmark.id);
                    } else {
                        info!("Deleted bookmark {} because it was removed from recents.", bookmark.id);
                    }
                }
            }

            page += 1;
        }

        sleep(SYNC_COOLDOWN_DURATION);
    }
}

pub async fn insert_all_bookmarks(mongodb: &MongoDB) -> Result<()> {
    let first_page = PixivBookmarks::get_page(1, "").await?;
    let total_pages = ((first_page.body.total as f64) / (PIXIV_BOOKMARKS_PER_PAGE as f64)).ceil() as i64;
    let mut page = total_pages;

    while page != 0 {
        info!("Inserting page {page}/{total_pages}...");

        let bookmarks = PixivBookmarks::get_page(page, "").await?;
        mongodb.bookmarks.insert_many(bookmarks.body.works).await?;

        page -= 1;
        sleep(INSERT_ALL_COOLDOWN_DURATION);
    }

    Ok(())
}
