mod error;
mod hotkey;
mod windows;

// ---------------------------------------------------------------------------------------------- //

use crate::{
    error::{AppError, AppResult},
    hotkey::{Controller, new_event_channel, select_backend_kind, start_event_task, start_runtime},
    windows::open_main_window,
};

use gpui::{App, AppContext, Application as GpuiApplication, Pixels, WindowBounds, px, size};
use gpui_component_assets::Assets as GpuiComponentAssets;

const WINDOW_TITLE: &str = "Hotkey Hold App";
const RESIZE_EDGE_SIZE: Pixels = px(6.0);
const RESIZE_CORNER_SIZE: Pixels = px(14.0);
const DEFAULT_START_WIDTH: f32 = 520.0;
const DEFAULT_START_HEIGHT: f32 = 600.0;
const MINIMUM_WIDTH: f32 = 360.0;
const MINIMUM_HEIGHT: f32 = 220.0;

fn main() {
    GpuiApplication::new()
        .with_assets(GpuiComponentAssets)
        .run(run_app);
}

fn run_app(app: &mut App) {
    gpui_component::init(app);

    if let Err(error) = start(app) {
        report_error(&error);
    }
}

fn start(app: &mut App) -> AppResult<()> {
    let backend_kind = select_backend_kind();
    let (sender, receiver) = new_event_channel();
    let controller = app.new(|cx| Controller::new(backend_kind, cx));
    let runtime = start_runtime(backend_kind, sender).map_err(AppError::hotkey_runtime)?;
    let event_task = start_event_task(controller.clone(), receiver, app);
    let window_closed_subscription = app.on_window_closed({
        let controller = controller.clone();
        move |app| {
            controller.update(app, |controller, cx| {
                controller.window_closed(cx);
            });
        }
    });

    controller.update(app, |controller, cx| {
        controller.install_runtime(runtime, event_task, window_closed_subscription, cx);
    });

    let window_bounds =
        WindowBounds::centered(size(px(DEFAULT_START_WIDTH), px(DEFAULT_START_HEIGHT)), app);

    open_main_window(app, controller, window_bounds).map_err(AppError::main_window)
}

fn report_error(error: &AppError) {
    eprintln!("{} {}", error.message(), error.location());
}
