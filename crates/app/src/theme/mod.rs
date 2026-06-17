use crate::THEME_DIRECTORY_PATH;

use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use std::path::PathBuf;

pub(crate) fn init(app: &mut App) {
    let theme_name = SharedString::from("Tokyo Night");

    ThemeRegistry::watch_dir(PathBuf::from(THEME_DIRECTORY_PATH), app, move |app| {
        if let Some(theme) = ThemeRegistry::global(app)
            .themes()
            .get(&theme_name)
            .cloned()
        {
            Theme::global_mut(app).apply_config(&theme);
            app.refresh_windows();
        }
    })
    .expect("failed to watch themes directory");
}
