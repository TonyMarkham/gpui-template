use crate::{
    DISCLOSURE_SLOT, INDENT_WIDTH, Icons, Item, ItemKind, ROW_HEIGHT, Row, TYPE_ICON_SLOT,
    build_rows, collect_default_expanded_ids, find_item, item_has_children,
};

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ListSizingBehavior, ParentElement,
    Render, StatefulInteractiveElement, Styled, UniformListScrollHandle, Window, div,
    prelude::FluentBuilder, px, uniform_list,
};
use gpui_component::{ActiveTheme, Icon, h_flex};
use std::{collections::HashSet, ops::Range};

pub struct TreeView {
    roots: Vec<Item>,
    rows: Vec<Row>,
    expanded_ids: HashSet<String>,
    selected_id: Option<String>,
    scroll_handle: UniformListScrollHandle,
    icons: Icons,
}

impl TreeView {
    pub fn new(roots: impl Into<Vec<Item>>) -> Self {
        let roots = roots.into();
        let mut expanded_ids = HashSet::new();
        collect_default_expanded_ids(&roots, &mut expanded_ids);

        let mut this = Self {
            roots,
            rows: Vec::new(),
            expanded_ids,
            selected_id: None,
            scroll_handle: UniformListScrollHandle::default(),
            icons: Icons::default(),
        };
        this.rebuild_rows();
        this
    }

    pub fn set_icons(&mut self, icons: Icons, cx: &mut Context<Self>) {
        self.icons = icons;
        cx.notify();
    }

    pub fn set_items(&mut self, roots: impl Into<Vec<Item>>, cx: &mut Context<Self>) {
        self.roots = roots.into();
        self.expanded_ids.clear();
        collect_default_expanded_ids(&self.roots, &mut self.expanded_ids);
        self.selected_id = None;
        self.rebuild_rows();
        cx.notify();
    }

    pub fn visible_len(&self) -> usize {
        self.rows.len()
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    pub fn selected_item(&self) -> Option<&Item> {
        self.selected_id
            .as_deref()
            .and_then(|id| find_item(&self.roots, id))
    }

    pub fn set_selected_id(&mut self, id: impl Into<String>, cx: &mut Context<Self>) {
        self.selected_id = Some(id.into());
        cx.notify();
    }

    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selected_id = None;
        cx.notify();
    }

    pub fn set_expanded(&mut self, id: impl AsRef<str>, expanded: bool, cx: &mut Context<Self>) {
        let id = id.as_ref();
        if expanded {
            self.expanded_ids.insert(id.to_owned());
        } else {
            self.expanded_ids.remove(id);
        }
        self.rebuild_rows();
        cx.notify();
    }

    fn select_id(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn toggle_id(&mut self, id: &str, cx: &mut Context<Self>) {
        if !item_has_children(&self.roots, id) {
            return;
        }

        if self.expanded_ids.contains(id) {
            self.expanded_ids.remove(id);
        } else {
            self.expanded_ids.insert(id.to_owned());
        }

        self.rebuild_rows();
        cx.notify();
    }

    fn rebuild_rows(&mut self) {
        self.rows.clear();
        build_rows(&self.roots, 0, &self.expanded_ids, &mut self.rows);
    }

    fn render_entries(
        &mut self,
        visible_range: Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut items = Vec::with_capacity(visible_range.len());

        for ix in visible_range {
            let Some(row) = self.rows.get(ix).cloned() else {
                continue;
            };

            items.push(self.render_row(ix, row, cx));
        }

        items
    }

    fn render_row(&self, ix: usize, row: Row, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selected_id.as_deref() == Some(row.id.as_str());
        let text_color = if selected {
            cx.theme().accent_foreground
        } else {
            cx.theme().foreground
        };
        let bg = if selected {
            cx.theme().accent
        } else {
            cx.theme().transparent
        };
        let row_id = row.id.clone();
        let disclosure_id = row.id.clone();
        let disclosure_icon = if row.expanded {
            self.icons.expanded.clone()
        } else {
            self.icons.collapsed.clone()
        };
        let type_icon = match row.icon {
            ItemKind::File => self.icons.file.clone(),
            ItemKind::Folder if row.expanded => self.icons.folder_open.clone(),
            ItemKind::Folder => self.icons.folder.clone(),
        };

        div()
            .id(ix)
            .h(px(ROW_HEIGHT))
            .w_full()
            .flex_none()
            .cursor_pointer()
            .items_center()
            .text_sm()
            .line_height(px(ROW_HEIGHT))
            .text_color(text_color)
            .bg(bg)
            .hover({
                let hover = cx.theme().list_hover;
                move |this| {
                    if selected { this } else { this.bg(hover) }
                }
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_id(row_id.clone(), cx);
            }))
            .child(
                h_flex()
                    .h_full()
                    .w_full()
                    .items_center()
                    .overflow_hidden()
                    .pl(px(4.) + px(INDENT_WIDTH) * row.depth)
                    .pr_2()
                    .child(
                        div()
                            .id(("tree-disclosure", ix))
                            .size(px(DISCLOSURE_SLOT))
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .when(row.has_children, |this| {
                                this.cursor_pointer()
                                    .child(Icon::new(disclosure_icon).size_3())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle_id(&disclosure_id, cx);
                                        cx.stop_propagation();
                                    }))
                            }),
                    )
                    .child(
                        div()
                            .size(px(TYPE_ICON_SLOT))
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .text_color(cx.theme().muted_foreground)
                            .child(Icon::new(type_icon).size_3p5()),
                    )
                    .child(div().ml_1().min_w_0().truncate().child(row.label.clone())),
            )
            .into_any_element()
    }
}

impl Render for TreeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().id("tree-view").size_full().child(
            uniform_list(
                "tree-view-rows",
                self.rows.len(),
                cx.processor(Self::render_entries),
            )
            .track_scroll(self.scroll_handle.clone())
            .with_sizing_behavior(ListSizingBehavior::Auto)
            .size_full(),
        )
    }
}
