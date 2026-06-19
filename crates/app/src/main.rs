mod error;
mod theme;
mod windows;

// ---------------------------------------------------------------------------------------------- //

use crate::{
    error::AppError, theme::init as AppThemeInit, windows::open_main_window as app_open_main_window,
};

use gpui::{App, Pixels, WindowBounds, px, size};
use gpui_component_assets::Assets as GpuiComponentAssets;

const WINDOW_TITLE: &str = "Template Desktop App";
const THEME_DIRECTORY_PATH: &str = "./themes";
const DEFAULT_THEME_NAME: &str = "Tokyo Night";
const RESIZE_EDGE_SIZE: Pixels = px(6.0);
const RESIZE_CORNER_SIZE: Pixels = px(14.0);
const DEFAULT_START_WIDTH: f32 = 800.0;
const DEFAULT_START_HEIGHT: f32 = 600.0;
const MINIMUM_WIDTH: f32 = 300.0;
const MINIMUM_HEIGHT: f32 = 200.0;

fn main() {
    gpui_platform::application()
        .with_assets(GpuiComponentAssets)
        .run(run_app);
}

fn run_app(app: &mut App) {
    gpui_component::init(app);
    if let Err(error) = AppThemeInit(app) {
        report_error(&error);
    }

    let window_bounds =
        WindowBounds::centered(size(px(DEFAULT_START_WIDTH), px(DEFAULT_START_HEIGHT)), app);

    if let Err(error) = app_open_main_window(app, window_bounds) {
        report_error(&error);
    }
}

fn report_error(error: &AppError) {
    eprintln!("{} {}", error.message(), error.location());
}
