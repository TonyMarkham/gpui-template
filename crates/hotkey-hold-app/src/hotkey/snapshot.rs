#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub(crate) hotkey_label: &'static str,
    pub(crate) backend_label: &'static str,
    pub(crate) is_hotkey_down: bool,
    pub(crate) popup_open: bool,
    pub(crate) status: String,
}
