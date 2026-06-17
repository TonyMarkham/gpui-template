use crate::ItemKind;

use gpui::SharedString;

#[derive(Clone, Debug)]
pub struct Item {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: ItemKind,
    pub(crate) expanded: bool,
    pub(crate) children: Vec<Item>,
}

impl Item {
    pub fn file(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label, ItemKind::File)
    }

    pub fn folder(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id, label, ItemKind::Folder)
    }

    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: ItemKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon,
            expanded: false,
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: Item) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Item>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn icon(mut self, icon: ItemKind) -> Self {
        self.icon = icon;
        self
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }

    pub fn children_ref(&self) -> &[Item] {
        &self.children
    }

    pub fn is_expanded_by_default(&self) -> bool {
        self.expanded
    }
}
