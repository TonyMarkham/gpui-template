pub(crate) mod error;
pub(crate) mod theme;
pub(crate) mod windows;

// ---------------------------------------------------------------------------------------------- //

use crate::{
    error::app_error::AppError, theme::init as init_theme, windows::main_window::MainWindow,
};

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;
use gpui::{AppContext, WindowBounds, WindowOptions, px, size};
use gpui_component::{Root, TitleBar};

pub(crate) const WINDOW_TITLE: &str = "Template Desktop App";
pub(crate) const THEME_DIRECTORY_PATH: &str = "./themes";
pub(crate) const DEFAULT_THEME_NAME: &str = "Tokyo Night";

fn main() {
    let app = gpui::Application::new().with_assets(gpui_component_assets::Assets);

    app.run(move |app| {
        gpui_component::init(app);
        if let Err(error) = init_theme(app) {
            report_error(&error);
        }

        let window_bounds = WindowBounds::centered(size(px(800.), px(500.)), app);

        app.spawn(async move |async_app| {
            let window_options = WindowOptions {
                window_bounds: Some(window_bounds),
                titlebar: Some(TitleBar::title_bar_options()),
                #[cfg(target_os = "linux")]
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            };

            if let Err(error) = async_app.open_window(window_options, |window, window_app| {
                window.set_window_title(WINDOW_TITLE);
                let main_window = window_app.new(|_| MainWindow);
                window_app.new(|root_context| Root::new(main_window, window, root_context))
            }) {
                report_error(&AppError::unexpected(format!(
                    "Failed to open desktop app window: {error}"
                )));
            }
        })
        .detach();
    });
}

fn report_error(error: &AppError) {
    eprintln!("{} {}", error.message(), error.location());
}
