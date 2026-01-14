use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

use crate::state::AppState;
use super::components::{AssetThumbnail, TimelineTrack, Toolbar};

#[component]
pub fn AssetBrowser(state: Signal<AppState>) -> Element {
    let assets = use_resource(move || async move {
        let mut app_state = state.write();
        
        // Initialize asset manager if needed
        if app_state.assets.read().is_empty() {
            // You could load some sample assets here
        }
        
        app_state.assets.read().values().cloned().collect::<Vec<_>>()
    });

    let mut import_in_progress = use_signal(|| false);

    rsx! {
        div {
            class: "asset-browser",
            div {
                class: "browser-header",
                h3 { "Media Library" }
                button {
                    class: "import-btn",
                    disabled: *import_in_progress.read(),
                    onclick: move |_| {
                        spawn({
                            let mut import_in_progress = import_in_progress;
                            let state = state.clone();
                            
                            async move {
                                import_in_progress.set(true);
                                
                                if let Some(file) = AsyncFileDialog::new()
                                    .add_filter("Media files", &["mp4", "avi", "mov", "mp3", "wav", "jpg", "png", "gif"])
                                    .add_filter("Video files", &["mp4", "avi", "mov", "mkv", "webm"])
                                    .add_filter("Audio files", &["mp3", "wav", "flac", "ogg"])
                                    .add_filter("Image files", &["jpg", "jpeg", "png", "gif", "bmp"])
                                    .pick_file()
                                    .await 
                                {
                                    let path = file.path().to_path_buf();
                                    
                                    // TODO: Actually import the file using AssetManager
                                    println!("Importing file: {:?}", path);
                                    
                                    // Simulate import
                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                }
                                
                                import_in_progress.set(false);
                            }
                        });
                    },
                    if *import_in_progress.read() {
                        "Importing..."
                    } else {
                        "📁 Import Media"
                    }
                }
            }
            div {
                class: "asset-categories",
                button { class: "category-btn active", "All" }
                button { class: "category-btn", "Video" }
                button { class: "category-btn", "Audio" }
                button { class: "category-btn", "Images" }
            }
            div {
                class: "asset-grid",
                if assets.read().is_empty() {
                    div {
                        class: "empty-assets",
                        "No media imported yet. Click 'Import Media' to add files."
                    }
                } else {
                    for asset in assets.read().iter() {
                        AssetThumbnail {
                            asset: asset.clone(),
                            on_drag_start: move |_| {},
                            on_click: move |_| {},
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ExportPanel(state: Signal<AppState>) -> Element {
    let mut export_settings = use_signal(|| ExportSettings::default());
    let mut export_in_progress = use_signal(|| false);
    let mut export_status = use_signal(|| None::<String>);

    rsx! {
        div {
            class: "export-panel",
            h3 { "Export Video" }
            div {
                class: "export-settings",
                div {
                    class: "setting-group",
                    label { "Resolution:" }
                    select {
                        onchange: move |e| {
                            let mut settings = export_settings.write();
                            match e.value().as_str() {
                                "480p" => { settings.width = 854; settings.height = 480; }
                                "720p" => { settings.width = 1280; settings.height = 720; }
                                "1080p" => { settings.width = 1920; settings.height = 1080; }
                                "4k" => { settings.width = 3840; settings.height = 2160; }
                                _ => {}
                            }
                        },
                        option { value: "480p", "480p (SD)" }
                        option { value: "720p", "720p (HD)" }
                        option { value: "1080p", selected: true, "1080p (Full HD)" }
                        option { value: "4k", "4K (Ultra HD)" }
                    }
                }
                div {
                    class: "setting-group",
                    label { "Frame Rate:" }
                    select {
                        onchange: move |e| {
                            export_settings.write().fps = e.value().parse().unwrap_or(30);
                        },
                        option { value: "24", "24 fps" }
                        option { value: "30", selected: true, "30 fps" }
                        option { value: "60", "60 fps" }
                    }
                }
                div {
                    class: "setting-group",
                    label { "Format:" }
                    select {
                        onchange: move |e| {
                            export_settings.write().format = e.value();
                        },
                        option { value: "mp4", selected: true, "MP4 (H.264)" }
                        option { value: "mov", "MOV (ProRes)" }
                        option { value: "webm", "WebM (VP9)" }
                    }
                }
                div {
                    class: "setting-group",
                    label { "Quality:" }
                    input {
                        r#type: "range",
                        min: "0",
                        max: "51",
                        value: "23",
                        oninput: move |e| {
                            export_settings.write().quality = e.value().parse().unwrap_or(23);
                        }
                    }
                    span { "CRF: {export_settings.read().quality}" }
                }
            }
            button {
                class: "export-btn",
                disabled: *export_in_progress.read(),
                onclick: move |_| {
                    spawn({
                        let state = state.clone();
                        let export_settings = export_settings.clone();
                        let mut export_in_progress = export_in_progress;
                        let mut export_status = export_status;
                        
                        async move {
                            export_in_progress.set(true);
                            export_status.set(Some("Preparing export...".to_string()));
                            
                            if let Some(file) = AsyncFileDialog::new()
                                .add_filter("MP4 Video", &["mp4"])
                                .add_filter("MOV Video", &["mov"])
                                .add_filter("WebM Video", &["webm"])
                                .set_file_name("exported_video.mp4")
                                .save_file()
                                .await
                            {
                                let settings = export_settings.read();
                                export_status.set(Some("Exporting video...".to_string()));
                                
                                match state.read().export_project(file.path().to_path_buf()).await {
                                    Ok(_) => {
                                        export_status.set(Some("✅ Export completed successfully!".to_string()));
                                    }
                                    Err(e) => {
                                        export_status.set(Some(format!("❌ Export failed: {}", e)));
                                    }
                                }
                            } else {
                                export_status.set(None);
                            }
                            
                            export_in_progress.set(false);
                        }
                    });
                },
                if *export_in_progress.read() {
                    "⏳ Exporting..."
                } else {
                    "🚀 Export Video"
                }
            }
            if let Some(status) = export_status.read().as_ref() {
                div {
                    class: "export-status",
                    "{status}"
                }
            }
        }
    }
}

#[derive(Clone)]
struct ExportSettings {
    width: u32,
    height: u32,
    fps: u32,
    format: String,
    quality: u32,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            format: "mp4".to_string(),
            quality: 23,
        }
    }
}