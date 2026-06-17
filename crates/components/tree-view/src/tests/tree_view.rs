use crate::{Item, TreeView};

#[test]
fn flattens_only_expanded_branches() {
    let tree = TreeView::new(vec![
        Item::folder("root", "root")
            .expanded(true)
            .child(Item::file("root/a.rs", "a.rs"))
            .child(
                Item::folder("root/src", "src").child(Item::file("root/src/main.rs", "main.rs")),
            ),
    ]);

    assert_eq!(tree.visible_len(), 3);
}

#[test]
fn default_expanded_state_is_collected_recursively() {
    let tree = TreeView::new(vec![
        Item::folder("root", "root").expanded(true).child(
            Item::folder("root/src", "src")
                .expanded(true)
                .child(Item::file("root/src/main.rs", "main.rs")),
        ),
    ]);

    assert_eq!(tree.visible_len(), 3);
}
