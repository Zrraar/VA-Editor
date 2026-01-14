use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::state::AppState;
use crate::ui::panels::{AssetBrowser, TimelinePanel, PreviewPanel, PropertiesPanel, ExportPanel};
use crate::ui::components::Toolbar;

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    #[layout(MainLayout)]
    #[nest("/")]
    #[layout(EditorLayout)]
    #[end_layout]
    #[child]
    Editor {},
    
    #[route("/export")]
    #[layout(MainLayout)]
    Export {},
}

#[component]
pub fn VideoEditorApp() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn MainLayout() -> Element {
    rsx! {
        div {
            class: "main-layout",
            Outlet::<Route> {}
        }
    }
}

#[component]
fn EditorLayout() -> Element {
    let mut state = use_signal(AppState::default);
    let navigator = use_navigator();

    rsx! {
        div {
            class: "editor-layout",
            Toolbar { state }
            div {
                class: "main-content",
                div {
                    class: "left-panel",
                    AssetBrowser { state }
                    PropertiesPanel { state }
                    button {
                        class: "export-sidebar-btn",
                        onclick: move |_| navigator.push(Route::Export {}),
                        "Export Video"
                    }
                }
                div {
                    class: "center-panel",
                    PreviewPanel { state }
                    TimelinePanel { state }
                }
            }
        }
    }
}

#[component]
fn Export() -> Element {
    let state = use_signal(AppState::default);
    let navigator = use_navigator();

    rsx! {
        div {
            class: "export-page",
            div {
                class: "export-header",
                h1 { "Export Video" }
                button {
                    class: "back-btn",
                    onclick: move |_| navigator.go_back(),
                    "← Back to Editor"
                }
            }
            ExportPanel { state }
        }
    }
}