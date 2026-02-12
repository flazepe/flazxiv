use crate::{CONFIG, REQWEST, USER_AGENT};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PixivBookmarkTags {
    pub body: PixivUserTagsBody,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PixivUserTagsBody {
    pub public: Vec<PixivBookmarkTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PixivBookmarkTag {
    #[serde(rename(serialize = "_id"), alias = "_id")]
    pub tag: String,
    pub cnt: u32,
}

impl PixivBookmarkTags {
    pub async fn get() -> Result<Self> {
        Ok(REQWEST
            .get(format!("https://www.pixiv.net/ajax/user/{}/illusts/bookmark/tags", CONFIG.pixiv_user_id))
            .header("user-agent", USER_AGENT)
            .header("cookie", format!("PHPSESSID={}", CONFIG.pixiv_phpsessid))
            .send()
            .await?
            .json()
            .await?)
    }
}
