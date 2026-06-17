use gpui_component::IconName;

#[derive(Clone)]
pub struct Icons {
    pub expanded: IconName,
    pub collapsed: IconName,
    pub folder: IconName,
    pub folder_open: IconName,
    pub file: IconName,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            expanded: IconName::ChevronDown,
            collapsed: IconName::ChevronRight,
            folder: IconName::Folder,
            folder_open: IconName::FolderOpen,
            file: IconName::File,
        }
    }
}
