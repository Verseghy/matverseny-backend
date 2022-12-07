use serde::{de, Deserializer};
use uuid::Uuid;

struct Visitor;

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Uuid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a uuid prefixed with `UserID-`")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Some(id) = v.strip_prefix("UserID-") {
            if let Ok(id) = Uuid::parse_str(id) {
                Ok(id)
            } else {
                Err(de::Error::invalid_value(de::Unexpected::Str(id), &self))
            }
        } else {
            Err(de::Error::invalid_value(de::Unexpected::Str(v), &self))
        }
    }
}

pub fn deserialize_subject<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(Visitor)
}
