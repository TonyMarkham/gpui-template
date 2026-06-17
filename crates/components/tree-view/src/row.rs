use crate::ItemKind;

use gpui::SharedString;

#[derive(Clone, Debug)]
pub struct Row {
    pub(crate) id: String,
    pub(crate) label: SharedString,
    pub(crate) icon: ItemKind,
    pub(crate) depth: usize,
    pub(crate) has_children: bool,
    pub(crate) expanded: bool,
}
