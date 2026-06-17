use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_component::{ActiveTheme as _, Root, TitleBar, v_flex};
use tree_view::{Item, TreeView};

#[cfg(target_os = "linux")]
use gpui::WindowDecorations;

const WINDOW_TITLE: &str = "Tree View Demo";

struct TreeViewDemo {
    tree: Entity<TreeView>,
}

impl TreeViewDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            tree: cx.new(|_| TreeView::new(sample_tree())),
        }
    }
}

impl Render for TreeViewDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(TitleBar::new().child(WINDOW_TITLE))
            .child(
                div()
                    .size_full()
                    .bg(cx.theme().background)
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(self.tree.clone()),
            )
    }
}

fn main() {
    let app = gpui::Application::new().with_assets(gpui_component_assets::Assets);

    app.run(move |app| {
        gpui_component::init(app);

        let window_bounds = WindowBounds::centered(size(px(420.), px(620.)), app);

        app.spawn(async move |async_app| {
            let window_options = WindowOptions {
                window_bounds: Some(window_bounds),
                titlebar: Some(TitleBar::title_bar_options()),
                #[cfg(target_os = "linux")]
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            };

            if let Err(error) = async_app.open_window(window_options, |window, cx| {
                window.set_window_title(WINDOW_TITLE);
                let demo = cx.new(TreeViewDemo::new);
                cx.new(|root_context| Root::new(demo, window, root_context))
            }) {
                eprintln!("Failed to open tree view demo window: {error}");
            }
        })
        .detach();
    });
}

fn sample_tree() -> Vec<Item> {
    vec![
        Item::folder("gpui-template", "gpui-template")
            .expanded(true)
            .child(
                Item::folder("gpui-template/.agents", ".agents").child(Item::file(
                    "gpui-template/.agents/config.toml",
                    "config.toml",
                )),
            )
            .child(
                Item::folder("gpui-template/.cargo", ".cargo")
                    .expanded(true)
                    .child(Item::file(
                        "gpui-template/.cargo/config.toml",
                        "config.toml",
                    )),
            )
            .child(
                Item::folder("gpui-template/crates", "crates")
                    .expanded(true)
                    .child(
                        Item::folder("gpui-template/crates/app", "app")
                            .expanded(true)
                            .child(
                                Item::folder("gpui-template/crates/app/src", "src")
                                    .expanded(true)
                                    .child(
                                        Item::folder("gpui-template/crates/app/src/error", "error")
                                            .child(Item::file(
                                                "gpui-template/crates/app/src/error/mod.rs",
                                                "mod.rs",
                                            ))
                                            .child(Item::file(
                                                "gpui-template/crates/app/src/error/app_error.rs",
                                                "app_error.rs",
                                            )),
                                    )
                                    .child(
                                        Item::folder("gpui-template/crates/app/src/theme", "theme")
                                            .child(Item::file(
                                                "gpui-template/crates/app/src/theme/mod.rs",
                                                "mod.rs",
                                            )),
                                    )
                                    .child(
                                        Item::folder(
                                            "gpui-template/crates/app/src/windows",
                                            "windows",
                                        )
                                        .child(Item::file(
                                            "gpui-template/crates/app/src/windows/main_window.rs",
                                            "main_window.rs",
                                        )),
                                    )
                                    .child(Item::file(
                                        "gpui-template/crates/app/src/main.rs",
                                        "main.rs",
                                    )),
                            )
                            .child(Item::file(
                                "gpui-template/crates/app/Cargo.toml",
                                "Cargo.toml",
                            )),
                    )
                    .child(
                        Item::folder("gpui-template/crates/components", "components")
                            .expanded(true)
                            .child(
                                Item::folder(
                                    "gpui-template/crates/components/tree-view",
                                    "tree-view",
                                )
                                .expanded(true)
                                .child(Item::file(
                                    "gpui-template/crates/components/tree-view/src/lib.rs",
                                    "lib.rs",
                                ))
                                .child(Item::file(
                                    "gpui-template/crates/components/tree-view/Cargo.toml",
                                    "Cargo.toml",
                                )),
                            ),
                    )
                    .child(
                        Item::folder("gpui-template/crates/tree-view-app", "tree-view-app")
                            .expanded(true)
                            .child(Item::file(
                                "gpui-template/crates/tree-view-app/src/main.rs",
                                "main.rs",
                            ))
                            .child(Item::file(
                                "gpui-template/crates/tree-view-app/Cargo.toml",
                                "Cargo.toml",
                            )),
                    ),
            )
            .child(Item::folder("gpui-template/docs", "docs"))
            .child(Item::folder("gpui-template/themes", "themes"))
            .child(Item::file("gpui-template/.gitignore", ".gitignore"))
            .child(Item::file("gpui-template/Cargo.lock", "Cargo.lock"))
            .child(Item::file("gpui-template/Cargo.toml", "Cargo.toml"))
            .child(Item::file("gpui-template/README.md", "README.md"))
            .child(
                Item::folder("gpui-template/generated", "generated")
                    .expanded(true)
                    .children((0..300).map(|ix| {
                        Item::file(
                            format!("gpui-template/generated/item_{ix:03}.rs"),
                            format!("item_{ix:03}.rs"),
                        )
                    })),
            ),
    ]
}
