use crate::WINDOW_TITLE;

use gpui::{Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div};
use gpui_component::{ActiveTheme, TitleBar, v_flex};

pub(crate) struct MainWindow;

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, context: &mut Context<Self>) -> impl IntoElement {
        let title_bar = TitleBar::new();
        #[cfg(not(target_os = "macos"))]
        let title_bar = title_bar.child(WINDOW_TITLE);

        v_flex().size_full().child(title_bar).child(
            div()
                .id("window-body")
                .size_full()
                .bg(context.theme().background),
        )
    }
}
