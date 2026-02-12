use anyhow::Result;
use serde::{
    Deserialize, Deserializer,
    de::{SeqAccess, Visitor},
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::read_to_string,
};
use toml::from_str;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub pixiv_user_id: u32,
    pub pixiv_phpsessid: SensitiveString,
    pub mongodb_uri: SensitiveString,

    #[serde(default, deserialize_with = "deserialize_bookmark_tag_mappings")]
    pub bookmark_tag_mappings: HashMap<String, Vec<String>>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_string = read_to_string("config.toml")?;
        let config = from_str(&config_string)?;
        info!("Successfully loaded config: {config:#?}");
        Ok(config)
    }
}

#[derive(Deserialize)]
pub struct SensitiveString(String);

impl Display for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Debug for SensitiveString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "<redacted>")
    }
}

fn deserialize_bookmark_tag_mappings<'de, D: Deserializer<'de>>(deserializer: D) -> Result<HashMap<String, Vec<String>>, D::Error> {
    struct BookmarkTagMappingsVisitor;

    impl<'de> Visitor<'de> for BookmarkTagMappingsVisitor {
        type Value = HashMap<String, Vec<String>>;

        fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
            formatter.write_str("an array of tuples containing a string and an array of strings")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut hashmap = HashMap::new();

            while let Some((tag, pixiv_tags)) = seq.next_element::<(String, Vec<String>)>()? {
                hashmap.insert(tag, pixiv_tags);
            }

            Ok(hashmap)
        }
    }

    deserializer.deserialize_seq(BookmarkTagMappingsVisitor)
}
