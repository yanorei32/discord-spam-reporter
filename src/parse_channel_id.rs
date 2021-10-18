use serde::{self, Deserialize, Deserializer};
use serenity::model::id::ChannelId;

pub fn deserialize<'de, D>(deserializer: D) -> Result<ChannelId, D::Error>
where
    D: Deserializer<'de>,
{
    let i = u64::deserialize(deserializer)?;
    Ok(ChannelId(i))
}
