use crate::hotkey::{BackendKind, Controller};

#[cfg(target_os = "linux")]
use std::process::Command;

use anyhow::{Context as _, Result};
use gpui::{
    App, AppContext, Bounds, Context, DisplayId, IntoElement, ParentElement, Pixels, Render,
    Styled, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind,
    WindowOptions, div, point, px, rgb, rgba, size,
};
use gpui_component::StyledExt;

#[cfg(all(target_os = "linux", feature = "wayland"))]
use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;

const HOTKEY_WINDOW_TITLE: &str = "Hotkey Overlay";
const HOTKEY_WINDOW_WIDTH: f32 = 360.0;
const HOTKEY_WINDOW_HEIGHT: f32 = 136.0;
const HOTKEY_WINDOW_BOTTOM_OFFSET: f32 = 120.0;

pub(crate) struct HotkeyWindow {
    backend_kind: BackendKind,
    is_visible: bool,
}

impl HotkeyWindow {
    pub(crate) fn new(backend_kind: BackendKind, _: &mut Context<Self>) -> Self {
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
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xcbd5e1))
                    .child(format!("Release {} to close", Controller::hotkey_label())),
            )
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
    backend_kind: BackendKind,
) -> Result<WindowHandle<HotkeyWindow>> {
    let placement = hotkey_window_placement(app);

    #[cfg(all(target_os = "linux", feature = "wayland"))]
    if app.compositor_name() == "Wayland" {
        return open_hotkey_window_with_kind(
            app,
            backend_kind,
            layer_shell_window_kind(placement.layer_shell_margin),
            placement,
        )
        .context("open Wayland layer-shell hotkey overlay");
    }

    open_hotkey_window_with_kind(app, backend_kind, WindowKind::PopUp, placement)
}

fn open_hotkey_window_with_kind(
    app: &mut App,
    backend_kind: BackendKind,
    kind: WindowKind,
    placement: HotkeyWindowPlacement,
) -> Result<WindowHandle<HotkeyWindow>> {
    let options = hotkey_window_options(kind, placement);

    app.open_window(options, move |window, app| {
        window.set_window_title(HOTKEY_WINDOW_TITLE);
        app.new(|cx| HotkeyWindow::new(backend_kind, cx))
    })
}

fn hotkey_window_options(kind: WindowKind, placement: HotkeyWindowPlacement) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(placement.window_bounds),
        titlebar: None,
        focus: false,
        show: true,
        kind,
        is_resizable: false,
        is_minimizable: false,
        display_id: placement.display_id,
        window_background: WindowBackgroundAppearance::Transparent,
        #[cfg(target_os = "linux")]
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    }
}

#[derive(Clone, Copy)]
struct HotkeyWindowPlacement {
    window_bounds: WindowBounds,
    display_id: Option<DisplayId>,
    #[cfg(all(target_os = "linux", feature = "wayland"))]
    layer_shell_margin: (Pixels, Pixels, Pixels, Pixels),
}

fn hotkey_window_placement(app: &App) -> HotkeyWindowPlacement {
    let window_size = size(px(HOTKEY_WINDOW_WIDTH), px(HOTKEY_WINDOW_HEIGHT));
    let bottom_offset = px(HOTKEY_WINDOW_BOTTOM_OFFSET);
    let Some(display) = app.primary_display() else {
        return HotkeyWindowPlacement {
            window_bounds: WindowBounds::Windowed(Bounds::new(
                point(px(0.0), px(0.0)),
                window_size,
            )),
            display_id: None,
            #[cfg(all(target_os = "linux", feature = "wayland"))]
            layer_shell_margin: (px(0.0), px(0.0), bottom_offset, px(0.0)),
        };
    };

    let display_bounds = placement_display_bounds(app, display.bounds());
    let origin = point(
        display_bounds.origin.x + (display_bounds.size.width - window_size.width) / 2.0,
        display_bounds.origin.y + display_bounds.size.height - window_size.height - bottom_offset,
    );

    HotkeyWindowPlacement {
        window_bounds: WindowBounds::Windowed(Bounds::new(origin, window_size)),
        display_id: Some(display.id()),
        #[cfg(all(target_os = "linux", feature = "wayland"))]
        layer_shell_margin: (
            px(0.0),
            px(0.0),
            bottom_offset,
            origin.x - display_bounds.origin.x,
        ),
    }
}

#[cfg(target_os = "linux")]
fn placement_display_bounds(app: &App, display_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    if app.compositor_name() == "X11"
        && let Some(primary_monitor_bounds) = x11_primary_monitor_bounds(display_bounds)
    {
        return primary_monitor_bounds;
    }

    display_bounds
}

#[cfg(not(target_os = "linux"))]
fn placement_display_bounds(_: &App, display_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    display_bounds
}

#[cfg(target_os = "linux")]
fn x11_primary_monitor_bounds(display_bounds: Bounds<Pixels>) -> Option<Bounds<Pixels>> {
    let output = Command::new("xrandr")
        .arg("--query")
        .arg("--current")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let root_size = parse_xrandr_root_size(&stdout)?;
    let monitor = parse_xrandr_primary_monitor(&stdout)?;

    let display_width = f32::from(display_bounds.size.width);
    let display_height = f32::from(display_bounds.size.height);
    if display_width <= 0.0
        || display_height <= 0.0
        || root_size.width <= 0.0
        || root_size.height <= 0.0
        || monitor.width <= 0.0
        || monitor.height <= 0.0
    {
        return None;
    }

    let scale_x = root_size.width / display_width;
    let scale_y = root_size.height / display_height;
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return None;
    }

    Some(Bounds::new(
        point(
            display_bounds.origin.x + px(monitor.x / scale_x),
            display_bounds.origin.y + px(monitor.y / scale_y),
        ),
        size(px(monitor.width / scale_x), px(monitor.height / scale_y)),
    ))
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct XrandrSize {
    width: f32,
    height: f32,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct XrandrGeometry {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[cfg(target_os = "linux")]
fn parse_xrandr_root_size(output: &str) -> Option<XrandrSize> {
    for line in output.lines() {
        let Some((_, after_current)) = line.split_once(" current ") else {
            continue;
        };
        let mut parts = after_current.split_whitespace();
        let width = parts.next()?.parse::<f32>().ok()?;
        if parts.next()? != "x" {
            return None;
        }
        let height = parts.next()?.trim_end_matches(',').parse::<f32>().ok()?;

        return Some(XrandrSize { width, height });
    }

    None
}

#[cfg(target_os = "linux")]
fn parse_xrandr_primary_monitor(output: &str) -> Option<XrandrGeometry> {
    let mut first_connected_monitor = None;

    for line in output.lines() {
        let mut words = line.split_whitespace();
        if words.next().is_none() || words.next() != Some("connected") {
            continue;
        }

        let Some(geometry) = line
            .split_whitespace()
            .find_map(parse_xrandr_geometry_token)
        else {
            continue;
        };

        if line.split_whitespace().any(|word| word == "primary") {
            return Some(geometry);
        }

        if first_connected_monitor.is_none() {
            first_connected_monitor = Some(geometry);
        }
    }

    first_connected_monitor
}

#[cfg(target_os = "linux")]
fn parse_xrandr_geometry_token(token: &str) -> Option<XrandrGeometry> {
    let width_end = token.find('x')?;
    let width = token[..width_end].parse::<f32>().ok()?;
    let after_width = &token[width_end + 1..];
    let height_end = after_width.find(['+', '-'])?;
    let height = after_width[..height_end].parse::<f32>().ok()?;
    let coordinates = &after_width[height_end..];
    let y_start = coordinates[1..].find(['+', '-'])? + 1;
    let x = coordinates[..y_start].parse::<f32>().ok()?;
    let y = coordinates[y_start..]
        .trim_end_matches(',')
        .parse::<f32>()
        .ok()?;

    Some(XrandrGeometry {
        x,
        y,
        width,
        height,
    })
}

#[cfg(all(target_os = "linux", feature = "wayland"))]
fn layer_shell_window_kind(margin: (Pixels, Pixels, Pixels, Pixels)) -> WindowKind {
    WindowKind::LayerShell(LayerShellOptions {
        namespace: "hotkey-hold-overlay".to_string(),
        layer: Layer::Overlay,
        anchor: Anchor::BOTTOM | Anchor::LEFT,
        margin: Some(margin),
        keyboard_interactivity: KeyboardInteractivity::None,
        ..Default::default()
    })
}
