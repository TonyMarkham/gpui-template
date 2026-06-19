use crate::hotkey::{HotkeyBackendKind, HotkeyController};

use anyhow::Result;
use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Render, Styled, Window,
    WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions, div, px,
    rgb, rgba, size,
};
use gpui_component::StyledExt;

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;

const HOTKEY_WINDOW_TITLE: &str = "Hotkey Overlay";
const HOTKEY_WINDOW_WIDTH: f32 = 360.0;
const HOTKEY_WINDOW_HEIGHT: f32 = 136.0;

pub(crate) struct HotkeyWindow {
    backend_kind: HotkeyBackendKind,
    is_visible: bool,
}

impl HotkeyWindow {
    pub(crate) fn new(backend_kind: HotkeyBackendKind, _: &mut Context<Self>) -> Self {
        Self {
            backend_kind,
            is_visible: true,
        }
    }

    pub(crate) fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_visible = true;
        window.resize(size(px(HOTKEY_WINDOW_WIDTH), px(HOTKEY_WINDOW_HEIGHT)));
        cx.notify();
    }

    pub(crate) fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_visible = false;
        window.resize(size(px(1.0), px(1.0)));
        cx.notify();
    }
}

impl Render for HotkeyWindow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if !self.is_visible {
            return div().size_full().hidden();
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .bg(rgba(0x111827f2))
            .border_1()
            .border_color(rgb(0x38bdf8))
            .text_color(rgb(0xf8fafc))
            .child(div().text_2xl().font_semibold().child("Hotkey active"))
            .child(div().text_sm().text_color(rgb(0xcbd5e1)).child(format!(
                "Release {} to close",
                HotkeyController::hotkey_label()
            )))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x93c5fd))
                    .child(self.backend_kind.label()),
            )
    }
}

pub(crate) fn open_hotkey_window(
    app: &mut App,
    backend_kind: HotkeyBackendKind,
) -> Result<WindowHandle<HotkeyWindow>> {
    if backend_kind == HotkeyBackendKind::WaylandPortal {
        app.activate(false);
    }

    let options = hotkey_window_options(app);

    app.open_window(options, move |window, app| {
        window.set_window_title(HOTKEY_WINDOW_TITLE);
        app.new(|cx| HotkeyWindow::new(backend_kind, cx))
    })
}

fn hotkey_window_options(app: &mut App) -> WindowOptions {
    let window_bounds =
        WindowBounds::centered(size(px(HOTKEY_WINDOW_WIDTH), px(HOTKEY_WINDOW_HEIGHT)), app);

    WindowOptions {
        window_bounds: Some(window_bounds),
        titlebar: None,
        focus: false,
        show: true,
        kind: WindowKind::PopUp,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        #[cfg(target_os = "linux")]
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    }
}
