use anyhow::Error as AnyhowError;
use error_location::ErrorLocation;
use std::panic::Location;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("{message} {location}")]
    MainWindow {
        message: String,
        location: ErrorLocation,
    },

    #[error("{message} {location}")]
    HotkeyRuntime {
        message: String,
        location: ErrorLocation,
    },
}

impl AppError {
    #[track_caller]
    pub(crate) fn main_window(error: AnyhowError) -> Self {
        Self::MainWindow {
            message: format!("Failed to open main window: {error}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    #[track_caller]
    pub(crate) fn hotkey_runtime(error: AnyhowError) -> Self {
        Self::HotkeyRuntime {
            message: format!("Failed to start hotkey runtime: {error}"),
            location: ErrorLocation::from(Location::caller()),
        }
    }

    pub(crate) fn message(&self) -> &str {
        match self {
            Self::MainWindow { .. } => "Main Window Error",
            Self::HotkeyRuntime { .. } => "Hotkey Runtime Error",
        }
    }

    pub(crate) fn location(&self) -> ErrorLocation {
        match self {
            Self::MainWindow { location, .. } | Self::HotkeyRuntime { location, .. } => *location,
        }
    }
}
