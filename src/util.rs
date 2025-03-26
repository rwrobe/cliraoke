use serde::{Deserialize, Deserializer};
use serde_json::Number;

pub fn deserialize_u64<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Number::deserialize(deserializer)?;
    Ok(s.to_string())
}