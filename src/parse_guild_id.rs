use serde::{self, Deserialize, Deserializer};
use serenity::model::id::GuildId;

pub fn deserialize<'de, D>(deserializer: D) -> Result<GuildId, D::Error>
where
    D: Deserializer<'de>,
{
    let i = u64::deserialize(deserializer)?;
    Ok(GuildId(i))
}
