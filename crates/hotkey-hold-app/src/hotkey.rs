use crate::windows::{HotkeyWindow, open_hotkey_window};

use anyhow::{Result, anyhow};
use async_channel::{Receiver, Sender};
use gpui::{App, Context, Entity, Subscription, Task, WeakEntity, WindowHandle};
use std::{borrow::BorrowMut, fmt};

pub(crate) const HOTKEY_ID: &str = "hold-overlay";
const HOTKEY_LABEL: &str = "Ctrl+Alt+Space";
const WAYLAND_APP_ID: &str = "dev.gpui.HotkeyHoldApp";
const WAYLAND_PREFERRED_TRIGGER: &str = "CTRL+ALT+space";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HotkeyEvent {
    Pressed,
    Released,
}

#[derive(Clone, Debug)]
pub(crate) enum RuntimeEvent {
    Hotkey(HotkeyEvent),
    Status(String),
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HotkeyBackendKind {
    GlobalHotkey,
    WaylandPortal,
}

impl HotkeyBackendKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::GlobalHotkey => "global-hotkey",
            Self::WaylandPortal => "XDG portal global shortcuts",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HotkeySnapshot {
    pub(crate) hotkey_label: &'static str,
    pub(crate) backend_label: &'static str,
    pub(crate) is_hotkey_down: bool,
    pub(crate) popup_open: bool,
    pub(crate) status: String,
}

pub(crate) struct HotkeyController {
    runtime: Option<HotkeyRuntime>,
    event_task: Option<Task<()>>,
    window_closed_subscription: Option<Subscription>,
    backend_kind: HotkeyBackendKind,
    popup_window: Option<WindowHandle<HotkeyWindow>>,
    is_hotkey_down: bool,
    status: String,
}

impl HotkeyController {
    pub(crate) fn new(backend_kind: HotkeyBackendKind, _: &mut Context<Self>) -> Self {
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

    pub(crate) fn snapshot(&self) -> HotkeySnapshot {
        HotkeySnapshot {
            hotkey_label: HOTKEY_LABEL,
            backend_label: self.backend_kind.label(),
            is_hotkey_down: self.is_hotkey_down,
            popup_open: self.is_hotkey_down && self.popup_window.is_some(),
            status: self.status.clone(),
        }
    }

    pub(crate) fn install_runtime(
        &mut self,
        runtime: HotkeyRuntime,
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
            RuntimeEvent::Hotkey(HotkeyEvent::Pressed) => self.hotkey_pressed(cx),
            RuntimeEvent::Hotkey(HotkeyEvent::Released) => self.hotkey_released(cx),
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

pub(crate) fn new_event_channel() -> (Sender<RuntimeEvent>, Receiver<RuntimeEvent>) {
    async_channel::unbounded()
}

pub(crate) fn select_backend_kind() -> HotkeyBackendKind {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return HotkeyBackendKind::WaylandPortal;
        }
    }

    HotkeyBackendKind::GlobalHotkey
}

pub(crate) fn start_runtime(
    backend_kind: HotkeyBackendKind,
    sender: Sender<RuntimeEvent>,
) -> Result<HotkeyRuntime> {
    match backend_kind {
        HotkeyBackendKind::GlobalHotkey => native::start(sender),
        HotkeyBackendKind::WaylandPortal => wayland::start(sender),
    }
}

pub(crate) fn start_event_task(
    controller: Entity<HotkeyController>,
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
    controller: &WeakEntity<HotkeyController>,
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

pub(crate) enum HotkeyRuntime {
    Global { _runtime: native::GlobalRuntime },
    Wayland { _runtime: wayland::WaylandRuntime },
}

impl fmt::Debug for HotkeyRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global { .. } => f.write_str("HotkeyRuntime::Global"),
            Self::Wayland { .. } => f.write_str("HotkeyRuntime::Wayland"),
        }
    }
}

mod native {
    use super::{HOTKEY_LABEL, HotkeyEvent, RuntimeEvent};
    use anyhow::{Context as _, Result};
    use async_channel::Sender;
    use global_hotkey::{
        GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
        hotkey::{Code, HotKey, Modifiers},
    };

    pub(crate) struct GlobalRuntime {
        manager: GlobalHotKeyManager,
        hotkey: HotKey,
    }

    impl Drop for GlobalRuntime {
        fn drop(&mut self) {
            let _ = self.manager.unregister(self.hotkey);
            GlobalHotKeyEvent::set_event_handler::<fn(GlobalHotKeyEvent)>(None);
        }
    }

    pub(crate) fn start(sender: Sender<RuntimeEvent>) -> Result<super::HotkeyRuntime> {
        let manager = GlobalHotKeyManager::new().context("create global hotkey manager")?;
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
        let hotkey_id = hotkey.id();

        manager
            .register(hotkey)
            .with_context(|| format!("register {HOTKEY_LABEL}"))?;

        let status_sender = sender.clone();
        let _ = status_sender.try_send(RuntimeEvent::Status(format!(
            "Registered {HOTKEY_LABEL}; hold it to show the overlay"
        )));

        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.id() != hotkey_id {
                return;
            }

            let mapped_event = match event.state() {
                HotKeyState::Pressed => HotkeyEvent::Pressed,
                HotKeyState::Released => HotkeyEvent::Released,
            };

            let _ = sender.try_send(RuntimeEvent::Hotkey(mapped_event));
        }));

        Ok(super::HotkeyRuntime::Global {
            _runtime: GlobalRuntime { manager, hotkey },
        })
    }
}

#[cfg(target_os = "linux")]
mod wayland {
    use super::{
        HOTKEY_ID, HOTKEY_LABEL, HotkeyEvent, RuntimeEvent, WAYLAND_APP_ID,
        WAYLAND_PREFERRED_TRIGGER, anyhow, format_error_chain,
    };
    use anyhow::{Context as _, Result};
    use ashpd::{
        AppID, Error as AshpdError,
        desktop::{
            CreateSessionOptions, ResponseError,
            global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
        },
        register_host_app_with_connection, zbus,
    };
    use async_channel::Sender;
    use futures_util::StreamExt;
    use std::{
        env, fs, io,
        path::PathBuf,
        sync::mpsc::{self, RecvTimeoutError},
        thread,
        time::Duration,
    };

    const WAYLAND_DESKTOP_FILE_NAME: &str = "dev.gpui.HotkeyHoldApp.desktop";
    const WAYLAND_DESKTOP_ENTRY: &str = include_str!("../data/dev.gpui.HotkeyHoldApp.desktop");
    const HOST_APP_REGISTRATION_TIMEOUT: Duration = Duration::from_secs(5);

    pub(crate) struct WaylandRuntime {
        _thread: thread::JoinHandle<()>,
    }

    pub(crate) fn start(sender: Sender<RuntimeEvent>) -> Result<super::HotkeyRuntime> {
        let thread = thread::Builder::new()
            .name("wayland-global-shortcut-portal".to_string())
            .spawn(move || {
                if let Err(error) = run_portal(sender.clone()) {
                    let message = format_error_chain(&error);
                    let _ = sender.try_send(RuntimeEvent::Error(format!(
                        "Wayland global shortcut portal failed: {message}"
                    )));
                }
            })
            .context("spawn Wayland global shortcut portal thread")?;

        Ok(super::HotkeyRuntime::Wayland {
            _runtime: WaylandRuntime { _thread: thread },
        })
    }

    fn run_portal(sender: Sender<RuntimeEvent>) -> Result<()> {
        pollster::block_on(async move {
            let desktop_entry_path =
                ensure_desktop_entry().context("install XDG desktop file for Wayland portal")?;
            let _ = sender.try_send(RuntimeEvent::Status(format!(
                "Using desktop entry {}",
                desktop_entry_path.display()
            )));

            let connection = zbus::Connection::session()
                .await
                .context("connect to the session bus for Wayland global shortcuts")?;
            let app_id = WAYLAND_APP_ID
                .parse::<AppID>()
                .context("parse Wayland portal app id")?;

            let _ = sender.try_send(RuntimeEvent::Status(format!(
                "Registering {WAYLAND_APP_ID} with the XDG portal"
            )));

            register_host_app_with_bounded_wait(connection.clone(), app_id)?;

            let portal = GlobalShortcuts::with_connection(connection)
                .await
                .context("connect to org.freedesktop.portal.GlobalShortcuts")?;

            let session = portal
                .create_session(CreateSessionOptions::default())
                .await
                .context("create global shortcuts session")?;

            let shortcut = NewShortcut::new(HOTKEY_ID, "Show hold overlay")
                .preferred_trigger(Some(WAYLAND_PREFERRED_TRIGGER));

            let _ = sender.try_send(RuntimeEvent::Status(format!(
                "Requesting {HOTKEY_LABEL}; approve it in the portal dialog"
            )));

            let request = portal
                .bind_shortcuts(&session, &[shortcut], None, BindShortcutsOptions::default())
                .await
                .context("request Wayland global shortcut binding")?;

            let response = request.response().map_err(describe_bind_shortcuts_error)?;

            if !response
                .shortcuts()
                .iter()
                .any(|shortcut| shortcut.id() == HOTKEY_ID)
            {
                return Err(anyhow!(
                    "portal did not bind {HOTKEY_LABEL}; use the desktop shortcut dialog to allow it"
                ));
            }

            let _ = sender.try_send(RuntimeEvent::Status(format!(
                "Registered {HOTKEY_LABEL} through the Wayland portal"
            )));

            let activated = portal
                .receive_activated()
                .await
                .context("subscribe to Wayland shortcut activation")?
                .fuse();
            let deactivated = portal
                .receive_deactivated()
                .await
                .context("subscribe to Wayland shortcut deactivation")?
                .fuse();

            futures_util::pin_mut!(activated);
            futures_util::pin_mut!(deactivated);

            loop {
                futures_util::select! {
                    event = activated.next() => {
                        let Some(event) = event else {
                            break;
                        };

                        if event.shortcut_id() == HOTKEY_ID {
                            let _ = sender.try_send(RuntimeEvent::Hotkey(HotkeyEvent::Pressed));
                        }
                    }
                    event = deactivated.next() => {
                        let Some(event) = event else {
                            break;
                        };

                        if event.shortcut_id() == HOTKEY_ID {
                            let _ = sender.try_send(RuntimeEvent::Hotkey(HotkeyEvent::Released));
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn ensure_desktop_entry() -> Result<PathBuf> {
        let applications_dir = user_applications_dir()?;
        let desktop_entry_path = applications_dir.join(WAYLAND_DESKTOP_FILE_NAME);
        let desktop_entry = WAYLAND_DESKTOP_ENTRY.replace(
            "Exec=hotkey-hold-app",
            &format!("Exec={}", desktop_exec_value()?),
        );

        match fs::read_to_string(&desktop_entry_path) {
            Ok(existing_entry) if existing_entry == desktop_entry => return Ok(desktop_entry_path),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("read desktop entry {}", desktop_entry_path.display())
                });
            }
        }

        fs::create_dir_all(&applications_dir)
            .with_context(|| format!("create {}", applications_dir.display()))?;
        fs::write(&desktop_entry_path, desktop_entry)
            .with_context(|| format!("write {}", desktop_entry_path.display()))?;

        Ok(desktop_entry_path)
    }

    fn user_applications_dir() -> Result<PathBuf> {
        if let Some(data_home) = env::var_os("XDG_DATA_HOME")
            && !data_home.as_os_str().is_empty()
        {
            return Ok(PathBuf::from(data_home).join("applications"));
        }

        let home = env::var_os("HOME").context("resolve HOME for XDG desktop file installation")?;
        Ok(PathBuf::from(home).join(".local/share/applications"))
    }

    fn desktop_exec_value() -> Result<String> {
        let executable =
            env::current_exe().context("resolve current executable for XDG desktop entry")?;
        let executable = executable.to_string_lossy();
        let escaped = executable.replace('\\', "\\\\").replace('"', "\\\"");

        Ok(format!("\"{escaped}\""))
    }

    fn describe_bind_shortcuts_error(error: AshpdError) -> anyhow::Error {
        match error {
            AshpdError::Response(ResponseError::Cancelled) => {
                anyhow!("Wayland global shortcut binding was cancelled")
            }
            AshpdError::Response(ResponseError::Other) => anyhow!(
                "Wayland portal returned response code 2 (Other) while binding {HOTKEY_LABEL}. This matches known GNOME GlobalShortcuts backend failures; update xdg-desktop-portal-gnome/gnome-control-center or use a desktop portal with working BindShortcuts support"
            ),
            error => anyhow!(error).context("read Wayland global shortcut binding response"),
        }
    }

    fn register_host_app_with_bounded_wait(
        connection: zbus::Connection,
        app_id: AppID,
    ) -> Result<()> {
        let (result_sender, result_receiver) = mpsc::sync_channel(1);
        let _registration_thread = thread::Builder::new()
            .name("wayland-host-app-registration".to_string())
            .spawn(move || {
                let result =
                    pollster::block_on(register_host_app_with_connection(connection, app_id));
                let _ = result_sender.send(result);
            })
            .context("spawn XDG host app registration thread")?;

        match result_receiver.recv_timeout(HOST_APP_REGISTRATION_TIMEOUT) {
            Ok(result) => result.context("register host app with the XDG portal"),
            Err(RecvTimeoutError::Timeout) => Err(anyhow!(
                "XDG host app registration timed out after {} seconds; ensure {WAYLAND_DESKTOP_FILE_NAME} is installed and the desktop global shortcuts provider is responsive",
                HOST_APP_REGISTRATION_TIMEOUT.as_secs()
            )),
            Err(RecvTimeoutError::Disconnected) => Err(anyhow!(
                "XDG host app registration thread stopped before sending a result"
            )),
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod wayland {
    use anyhow::{Result, bail};
    use async_channel::Sender;

    use super::RuntimeEvent;

    pub(crate) struct WaylandRuntime;

    pub(crate) fn start(_: Sender<RuntimeEvent>) -> Result<super::HotkeyRuntime> {
        bail!("Wayland portal hotkeys are only available on Linux")
    }
}
