pub mod theme;
pub(crate) mod windows;
// ---------------------------------------------------------------------------------------------- //

use crate::{theme::init as init_theme, windows::main_window::MainWindow};

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;
use gpui::{AppContext, WindowBounds, WindowOptions, px, size};
use gpui_component::{Root, TitleBar};

pub(crate) const WINDOW_TITLE: &str = "Template Desktop App";
pub(crate) const THEME_DIRECTORY_PATH: &str = "./themes";

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |app| {
        gpui_component::init(app);
        init_theme(app);

        let window_bounds = WindowBounds::centered(size(px(800.), px(500.)), app);

        app.spawn(async move |async_app| {
            let window_options = WindowOptions {
                window_bounds: Some(window_bounds),
                titlebar: Some(TitleBar::title_bar_options()),
                #[cfg(target_os = "linux")]
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            };

            async_app
                .open_window(window_options, |window, window_app| {
                    window.set_window_title(WINDOW_TITLE);
                    let main_window = window_app.new(|_| MainWindow);
                    window_app.new(|root_context| Root::new(main_window, window, root_context))
                })
                .expect("failed to open desktop app window");
        })
        .detach();
    });
}
