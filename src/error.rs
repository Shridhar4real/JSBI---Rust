use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JSBIError {
    RangeError(String),
    SyntaxError(String),
    TypeError(String),
    GenericError(String),
}

impl fmt::Display for JSBIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSBIError::RangeError(msg) => write!(f, "RangeError: {}", msg),
            JSBIError::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            JSBIError::TypeError(msg) => write!(f, "TypeError: {}", msg),
            JSBIError::GenericError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for JSBIError {}
