use anyhow::Error as AnyhowError;
use error_location::ErrorLocation;
use std::{panic::Location, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{message} {location}")]
    MainWindow {
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
    pub(crate) fn main_window(e: AnyhowError) -> Self {
        Self::MainWindow {
            message: format!("Failed to open main window {e}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn theme_directory(path: PathBuf, e: AnyhowError) -> Self {
        Self::ThemeDirectory {
            path: path.clone(),
            message: format!("Failed to access theme directory: {e}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::MainWindow { .. } => "Main Window Error",
            Self::ThemeDirectory { .. } => "Theme Directory Error",
        }
    }

    pub fn location(&self) -> ErrorLocation {
        match self {
            Self::MainWindow { location, .. } | Self::ThemeDirectory { location, .. } => *location,
        }
    }
}
