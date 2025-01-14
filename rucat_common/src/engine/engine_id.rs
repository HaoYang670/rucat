use ::core::fmt::Display;
use ::std::{borrow::Cow, fmt};

use ::anyhow::anyhow;
use ::serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::error::{Result, RucatError};

/// Unique identifier for an engine.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Serialize)]
pub struct EngineId {
    id: Cow<'static, str>,
}

impl EngineId {
    pub fn new(id: Cow<'static, str>) -> Result<Self> {
        if id.is_empty() {
            Err(RucatError::not_allowed(anyhow!(
                "Engine id cannot be empty."
            )))
        } else {
            Ok(Self { id })
        }
    }
}

impl<'de> Deserialize<'de> for EngineId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(EngineIdVisitor)
    }
}

struct EngineIdVisitor;

impl Visitor<'_> for EngineIdVisitor {
    type Value = EngineId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a non-empty string representing an EngineId")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        EngineId::try_from(value.to_owned()).map_err(de::Error::custom)
    }
}

impl Display for EngineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl TryFrom<String> for EngineId {
    type Error = RucatError;
    fn try_from(id: String) -> Result<Self> {
        Self::new(Cow::Owned(id))
    }
}

impl TryFrom<&'static str> for EngineId {
    type Error = RucatError;
    fn try_from(id: &'static str) -> Result<Self> {
        Self::new(Cow::Borrowed(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::serde_json::json;

    #[test]
    fn engine_id_cannot_be_empty() {
        let result = EngineId::try_from("");
        assert!(result.is_err_and(|e| e
            .to_string()
            .starts_with("Not allowed: Engine id cannot be empty.")));
    }

    #[test]
    fn cannot_deserialize_empty_str_to_engine_id() {
        let result: std::result::Result<EngineId, _> = serde_json::from_value(json!(""));
        assert!(result.is_err_and(|e| e
            .to_string()
            .starts_with("Not allowed: Engine id cannot be empty.")));
    }

    #[test]
    fn deserialize_engine_id() -> anyhow::Result<()> {
        let result: EngineId = serde_json::from_value(json!("abc"))?;
        assert_eq!(result, EngineId::try_from("abc")?);
        Ok(())
    }
}
