use crate::{
    DEFAULT_THEME_NAME, THEME_DIRECTORY_PATH,
    error::{AppError, AppResult},
};

use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use std::path::PathBuf;

pub(crate) fn init(app: &mut App) -> AppResult<()> {
    let theme_directory = PathBuf::from(THEME_DIRECTORY_PATH);
    ThemeRegistry::watch_dir(theme_directory.clone(), app, on_load).map_err(|e| {
        AppError::theme_directory(
            theme_directory,
            format!("Failed to access theme directory: {e}"),
        )
    })?;

    Ok(())
}

fn on_load(app: &mut App) {
    let theme_name = SharedString::from(DEFAULT_THEME_NAME);

    if let Some(theme) = ThemeRegistry::global(app)
        .themes()
        .get(&theme_name)
        .cloned()
    {
        Theme::global_mut(app).apply_config(&theme);
        app.refresh_windows();
    }
}
