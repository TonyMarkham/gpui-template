use crate::{
    hotkey::{BackendKind, Event, HOTKEY_LABEL, Runtime, RuntimeEvent, Snapshot},
    windows::{HotkeyWindow, open_hotkey_window},
};

use gpui::{Context, Subscription, Task, WindowHandle};
use std::borrow::BorrowMut;

pub(crate) struct Controller {
    runtime: Option<Runtime>,
    event_task: Option<Task<()>>,
    window_closed_subscription: Option<Subscription>,
    backend_kind: BackendKind,
    popup_window: Option<WindowHandle<HotkeyWindow>>,
    is_hotkey_down: bool,
    status: String,
}

impl Controller {
    pub(crate) fn new(backend_kind: BackendKind, _: &mut Context<Self>) -> Self {
        Self {
            runtime: None,
            event_task: None,
            window_closed_subscription: None,
            backend_kind,
            popup_window: None,
            is_hotkey_down: false,
            status: format!("Starting {} backend", backend_kind.label()),
        }
    }

    pub(crate) fn hotkey_label() -> &'static str {
        HOTKEY_LABEL
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            hotkey_label: HOTKEY_LABEL,
            backend_label: self.backend_kind.label(),
            is_hotkey_down: self.is_hotkey_down,
            popup_open: self.is_hotkey_down && self.popup_window.is_some(),
            status: self.status.clone(),
        }
    }

    pub(crate) fn install_runtime(
        &mut self,
        runtime: Runtime,
        event_task: Task<()>,
        window_closed_subscription: Subscription,
        cx: &mut Context<Self>,
    ) {
        self.runtime = Some(runtime);
        self.event_task = Some(event_task);
        self.window_closed_subscription = Some(window_closed_subscription);
        cx.notify();
    }

    pub(crate) fn apply_runtime_event(&mut self, event: RuntimeEvent, cx: &mut Context<Self>) {
        match event {
            RuntimeEvent::Hotkey(Event::Pressed) => self.hotkey_pressed(cx),
            RuntimeEvent::Hotkey(Event::Released) => self.hotkey_released(cx),
            RuntimeEvent::Status(message) => {
                self.status = message;
                cx.notify();
            }
            RuntimeEvent::Error(message) => {
                self.status = message;
                cx.notify();
            }
        }
    }

    pub(crate) fn window_closed(&mut self, cx: &mut Context<Self>) {
        let popup_is_closed = self
            .popup_window
            .map(|handle| handle.update(cx, |_, _, _| ()).is_err())
            .unwrap_or(false);

        if popup_is_closed {
            self.popup_window = None;
            cx.notify();
        }

        let only_hidden_popup_remains = self.popup_window.is_some_and(|popup_window| {
            cx.windows()
                .into_iter()
                .all(|window| window.window_id() == popup_window.window_id())
        });

        if only_hidden_popup_remains {
            cx.quit();
        }
    }

    fn hotkey_pressed(&mut self, cx: &mut Context<Self>) {
        if self.is_hotkey_down {
            return;
        }

        self.is_hotkey_down = true;
        self.status = "Hotkey is down".to_string();

        if let Some(window) = self.popup_window
            && window
                .update(cx, |popup, window, cx| popup.show(window, cx))
                .is_err()
        {
            self.popup_window = None;
        }

        if self.popup_window.is_none() {
            match open_hotkey_window(cx.borrow_mut(), self.backend_kind) {
                Ok(window) => {
                    self.popup_window = Some(window);
                }
                Err(error) => {
                    self.status = format!("Failed to open hotkey window: {error}");
                }
            }
        }

        cx.notify();
    }

    fn hotkey_released(&mut self, cx: &mut Context<Self>) {
        self.is_hotkey_down = false;
        self.status = format!("Waiting for {}", HOTKEY_LABEL);

        if let Some(window) = self.popup_window
            && window
                .update(cx, |popup, window, cx| popup.hide(window, cx))
                .is_err()
        {
            self.popup_window = None;
        }

        cx.notify();
    }
}
