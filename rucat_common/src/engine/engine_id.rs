use ::core::fmt::Display;
use ::std::borrow::Cow;

use ::anyhow::anyhow;
use ::serde::{de, Deserialize, Deserializer, Serialize};

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

/// Almost same as the derive macro generated implementation, except empty string is not allowed.
impl<'de> Deserialize<'de> for EngineId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct _EngineId {
            id: String,
        }

        let _EngineId { id } = _EngineId::deserialize(deserializer).map_err(|_| {
            de::Error::custom(
                r#"Failed to deserialize EngineId, expect a map `{"id": <non empty string>}`"#,
            )
        })?;
        EngineId::try_from(id).map_err(de::Error::custom)
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
    fn display_engine_id() {
        let id = EngineId::try_from("abc").unwrap();
        assert_eq!(format!("{}", id), "abc");
    }

    #[test]
    fn cannot_deserialize_str_to_engine_id() {
        let result: std::result::Result<EngineId, _> = serde_json::from_value(json!("123"));
        print!("{:?}", result);
        assert!(result.is_err_and(|e| e
            .to_string()
            .eq("Failed to deserialize EngineId, expect a map `{\"id\": <non empty string>}`")));
    }

    #[test]
    fn deny_unknown_fields_in_deserialization() {
        let result: std::result::Result<EngineId, _> =
            serde_json::from_value(json!({"id": "123", "other": "456"}));
        print!("{:?}", result);
        assert!(result.is_err_and(|e| e
            .to_string()
            .eq("Failed to deserialize EngineId, expect a map `{\"id\": <non empty string>}`")));
    }

    #[test]
    fn cannot_deserialize_map_with_empty_str_to_engine_id() {
        let result: std::result::Result<EngineId, _> = serde_json::from_value(json!({"id": ""}));
        assert!(result.is_err_and(|e| e
            .to_string()
            .starts_with("Not allowed: Engine id cannot be empty.")));
    }

    #[test]
    fn deserialize_engine_id() -> anyhow::Result<()> {
        let result: EngineId = serde_json::from_value(json!({"id": "abc"}))?;
        assert_eq!(result, EngineId::try_from("abc")?);
        Ok(())
    }

    #[test]
    fn engine_id_ser_de_identity() -> anyhow::Result<()> {
        let id = EngineId::try_from("abc")?;
        let json = serde_json::to_value(&id)?;
        let id2: EngineId = serde_json::from_value(json)?;
        assert_eq!(id, id2);
        Ok(())
    }

    #[test]
    fn engine_id_de_ser_identity() -> anyhow::Result<()> {
        let json = json!({"id": "abc"});
        let id: EngineId = serde_json::from_value(json.clone())?;
        let json2 = serde_json::to_value(&id)?;
        assert_eq!(json, json2);
        Ok(())
    }
}
