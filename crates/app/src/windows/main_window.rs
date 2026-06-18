use crate::{
    MINIMUM_HEIGHT, MINIMUM_WIDTH, WINDOW_TITLE,
    error::{AppError, AppResult},
    windows::window_resize_handles,
};

use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_component::{ActiveTheme, Root, TitleBar, v_flex};

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;

pub(crate) struct MainWindow;

impl MainWindow {
    pub(crate) fn new(_: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title_bar = TitleBar::new();
        #[cfg(not(target_os = "macos"))]
        let title_bar = title_bar.child(WINDOW_TITLE);

        v_flex()
            .relative()
            .size_full()
            .child(title_bar)
            .child(
                div()
                    .id("window-body")
                    .size_full()
                    .bg(cx.theme().background),
            )
            .child(window_resize_handles(window))
    }
}

pub(crate) fn open_main_window(app: &mut App, window_bounds: WindowBounds) -> AppResult<()> {
    let options = WindowOptions {
        window_bounds: Some(window_bounds),
        window_min_size: Some(size(px(MINIMUM_WIDTH), px(MINIMUM_HEIGHT))),
        titlebar: Some(TitleBar::title_bar_options()),
        #[cfg(target_os = "linux")]
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    };

    match app.open_window(options, build_main_window) {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::main_window(e)),
    }
}

fn build_main_window(window: &mut Window, app: &mut App) -> Entity<Root> {
    window.set_window_title(WINDOW_TITLE);
    let main_window = app.new(MainWindow::new);
    app.new(|cx| Root::new(main_window, window, cx))
}
