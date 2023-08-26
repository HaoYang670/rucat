
pub type Result<T> = std::result::Result<T, RucatError>;

#[derive(Debug, PartialEq)]
pub enum RucatError {
  SerializationError(String),
}

impl From<serde_json::Error> for RucatError {
  fn from(value: serde_json::Error) -> Self {
    Self::SerializationError(value.to_string())
  }
}