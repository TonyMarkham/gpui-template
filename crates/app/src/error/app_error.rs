use error_location::ErrorLocation;
use std::{panic::Location, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{message} {location}")]
    Unexpected {
        message: String,
        location: ErrorLocation,
    },

    #[error("{message}: [{path}] {location}")]
    ThemeDirectory {
        path: PathBuf,
        message: String,
        location: ErrorLocation,
    },
}

impl AppError {
    #[track_caller]
    pub(crate) fn unexpected(message: impl Into<String>) -> Self {
        Self::Unexpected {
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn theme_directory(path: PathBuf, message: impl Into<String>) -> Self {
        Self::ThemeDirectory {
            path: path.clone(),
            message: message.into(),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Unexpected { .. } => "Unexpected Error",
            Self::ThemeDirectory { .. } => "Theme Directory Error",
        }
    }

    pub fn location(&self) -> ErrorLocation {
        match self {
            Self::Unexpected { location, .. } | Self::ThemeDirectory { location, .. } => *location,
        }
    }
}
