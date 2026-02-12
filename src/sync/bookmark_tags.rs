use crate::{MONGODB, pixiv::bookmark_tags::PixivBookmarkTags};
use anyhow::Result;
use std::{thread::sleep, time::Duration};
use tracing::{error, info};

const COOLDOWN_DURATION: Duration = Duration::from_secs(60);

pub async fn sync_bookmark_tags() -> Result<()> {
    let mongodb = MONGODB.get().unwrap();

    info!("Syncing bookmark tags...");

    loop {
        match PixivBookmarkTags::get().await {
            Ok(tags) => {
                if let Err(error) = mongodb.bookmark_tags.delete_all().await {
                    error!("An error occurred while trying to delete all bookmark tags: {error:?}");
                    sleep(COOLDOWN_DURATION);
                    continue;
                }

                if let Err(error) = mongodb.bookmark_tags.insert_many(tags.body.public).await {
                    error!("An error occurred while trying to insert bookmark tags: {error:?}");
                    sleep(COOLDOWN_DURATION);
                    continue;
                }
            },
            Err(error) => error!("An error occurred while trying to get bookmark tags: {error:?}"),
        }

        sleep(COOLDOWN_DURATION);
    }
}
