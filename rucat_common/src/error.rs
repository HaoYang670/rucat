pub type Result<T> = std::result::Result<T, RucatError>;

#[derive(Debug, PartialEq)]
pub enum RucatError {
    CannotChangeStorageLevelError,
    SerializationError(String),
    IllegalArgument(String),
    IOError(String),
    DataStoreError(String),
    Other(String),
}

impl<T> From<RucatError> for Result<T> {
    fn from(val: RucatError) -> Self {
        Result::Err(val)
    }
}

macro_rules! convert_to_rucat_error {
    ($err_ty: ty, $constructor: expr) => {
        impl From<$err_ty> for RucatError {
            fn from(value: $err_ty) -> Self {
                $constructor(value.to_string())
            }
        }
    };
}

convert_to_rucat_error!(std::io::Error, RucatError::IOError);
convert_to_rucat_error!(surrealdb::Error, RucatError::DataStoreError);
convert_to_rucat_error!(String, RucatError::Other);
