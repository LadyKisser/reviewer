use serde::{Deserialize, Deserializer, Serializer};
use time::OffsetDateTime;

pub mod datetime_format {
    use super::*;

    pub fn serialize<S>(
        date: &Option<OffsetDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => serializer.serialize_str(&date.to_string()),
            None => serializer.serialize_none(),
        }
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<OffsetDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Ok(Some(
                OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map_err(serde::de::Error::custom)?,
            )),
            None => Ok(None),
        }
    }
} 