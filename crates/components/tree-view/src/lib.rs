mod icons;
mod item;
mod item_kind;
mod row;
#[cfg(test)]
mod tests;
mod tree_view;

// ---------------------------------------------------------------------------------------------- //

pub use icons::Icons;
pub use item::Item;
pub use item_kind::ItemKind;
pub use row::Row;
pub use tree_view::TreeView;

// ---------------------------------------------------------------------------------------------- //

use std::collections::HashSet;

const ROW_HEIGHT: f32 = 22.;
const INDENT_WIDTH: f32 = 18.;
const DISCLOSURE_SLOT: f32 = 18.;
const TYPE_ICON_SLOT: f32 = 18.;

fn collect_default_expanded_ids(items: &[Item], expanded_ids: &mut HashSet<String>) {
    for item in items {
        if item.expanded {
            expanded_ids.insert(item.id.as_str().to_owned());
        }
        collect_default_expanded_ids(&item.children, expanded_ids);
    }
}

fn build_rows(items: &[Item], depth: usize, expanded_ids: &HashSet<String>, rows: &mut Vec<Row>) {
    for item in items {
        let id = item.id.as_str().to_owned();
        let has_children = !item.children.is_empty();
        let expanded = has_children && expanded_ids.contains(&id);

        rows.push(Row {
            id,
            label: item.label.clone(),
            icon: item.icon,
            depth,
            has_children,
            expanded,
        });

        if expanded {
            build_rows(&item.children, depth + 1, expanded_ids, rows);
        }
    }
}

fn find_item<'a>(items: &'a [Item], id: &str) -> Option<&'a Item> {
    for item in items {
        if item.id.as_str() == id {
            return Some(item);
        }

        if let Some(found) = find_item(&item.children, id) {
            return Some(found);
        }
    }

    None
}

fn item_has_children(items: &[Item], id: &str) -> bool {
    find_item(items, id).is_some_and(|item| !item.children.is_empty())
}
