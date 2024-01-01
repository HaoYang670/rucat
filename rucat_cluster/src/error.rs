pub type Result<T> = std::result::Result<T, RucatError>;

#[derive(Debug, PartialEq)]
pub enum RucatError {
    SerializationError(String),
    IOError(String),
    Other(String),
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
convert_to_rucat_error!(String, RucatError::Other);
