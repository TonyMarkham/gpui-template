mod backend_kind;
mod controller;
mod event;
mod global_runtime;
mod native;
mod runtime;
mod runtime_event;
mod snapshot;
mod wayland;
mod wayland_runtime;

// ---------------------------------------------------------------------------------------------- //

use anyhow::Result;
use async_channel::{Receiver, Sender};
use gpui::{App, Entity, Task, WeakEntity};

pub(crate) use backend_kind::BackendKind;
pub(crate) use controller::Controller;
pub(crate) use event::Event;
pub(crate) use global_runtime::GlobalRuntime;
pub(crate) use runtime::Runtime;
pub(crate) use runtime_event::RuntimeEvent;
pub(crate) use snapshot::Snapshot;
pub(crate) use wayland_runtime::WaylandRuntime;

// ---------------------------------------------------------------------------------------------- //

pub(crate) const HOTKEY_ID: &str = "hold-overlay";
const HOTKEY_LABEL: &str = "Ctrl+Alt+Space";
const WAYLAND_APP_ID: &str = "dev.gpui.HotkeyHoldApp";
const WAYLAND_PREFERRED_TRIGGER: &str = "CTRL+ALT+space";

pub(crate) fn new_event_channel() -> (Sender<RuntimeEvent>, Receiver<RuntimeEvent>) {
    async_channel::unbounded()
}

pub(crate) fn select_backend_kind() -> BackendKind {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return BackendKind::WaylandPortal;
        }
    }

    BackendKind::GlobalHotkey
}

pub(crate) fn start_runtime(
    backend_kind: BackendKind,
    sender: Sender<RuntimeEvent>,
) -> Result<Runtime> {
    match backend_kind {
        BackendKind::GlobalHotkey => native::start(sender),
        BackendKind::WaylandPortal => wayland::start(sender),
    }
}

pub(crate) fn start_event_task(
    controller: Entity<Controller>,
    receiver: Receiver<RuntimeEvent>,
    app: &mut App,
) -> Task<()> {
    let controller = controller.downgrade();

    app.spawn(move |cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            while let Ok(event) = receiver.recv().await {
                dispatch_runtime_event(&controller, event, &mut cx);
            }
        }
    })
}

fn dispatch_runtime_event(
    controller: &WeakEntity<Controller>,
    event: RuntimeEvent,
    cx: &mut gpui::AsyncApp,
) {
    let _ = controller.update(cx, |controller, cx| {
        controller.apply_runtime_event(event, cx);
    });
}

fn format_error_chain(error: &anyhow::Error) -> String {
    let mut message = String::new();

    for (index, cause) in error.chain().enumerate() {
        if index > 0 {
            message.push_str(": ");
        }
        message.push_str(&cause.to_string());
    }

    message
}
