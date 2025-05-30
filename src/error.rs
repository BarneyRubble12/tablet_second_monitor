use std::fmt;

#[derive(Debug)]
pub enum AppError {
    CaptureError(String),
    NetworkError(String),
    InputError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::CaptureError(e) => write!(f, "Capture error: {}", e),
            AppError::NetworkError(e) => write!(f, "Network error: {}", e),
            AppError::InputError(e) => write!(f, "Input error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::NetworkError(err.to_string())
    }
} 