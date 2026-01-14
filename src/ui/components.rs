use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::*;
use dioxus_free_icons::Icon;

use crate::state::{AppState, Asset, Clip, Transform};

#[component]
pub fn Toolbar(state: Signal<AppState>) -> Element {
    let mut app_state = state.write();

    rsx! {
        div {
            class: "toolbar",
            button {
                class: "toolbar-btn",
                onclick: move |_| {
                    app_state.save_snapshot();
                    app_state.is_playing = !app_state.is_playing;
                },
                if app_state.is_playing {
                    Icon { icon: FaPause }
                } else {
                    Icon { icon: FaPlay }
                }
            }
            button {
                class: "toolbar-btn",
                onclick: move |_| { app_state.undo(); },
                Icon { icon: FaUndo }
            }
            button {
                class: "toolbar-btn",
                onclick: move |_| { app_state.redo(); },
                Icon { icon: FaRedo }
            }
            button {
                class: "toolbar-btn",
                onclick: move |_| {
                    if !app_state.selected_clips.is_empty() {
                        app_state.delete_selected_clips();
                    }
                },
                Icon { icon: FaTrashAlt }
            }
            button {
                class: "toolbar-btn",
                onclick: move |_| {
                    if let Some(clip_id) = app_state.selected_clips.first() {
                        let timeline = app_state.timeline.borrow();
                        if let Some(clip) = timeline.tracks.iter()
                            .flat_map(|t| &t.clips)
                            .find(|c| &c.id == clip_id) 
                        {
                            let split_time = clip.start_time + clip.duration / 2.0;
                            drop(timeline);
                            app_state.split_clip(*clip_id, split_time);
                        }
                    }
                },
                Icon { icon: FaCut }
            }
            input {
                r#type: "range",
                min: "0",
                max: "100",
                value: "{app_state.volume * 50.0}",
                oninput: move |e| {
                    app_state.volume = e.value().parse::<f32>().unwrap_or(1.0) / 50.0;
                }
            }
        }
    }
}

#[component]
pub fn AssetThumbnail(
    asset: Asset,
    on_drag_start: EventHandler<Asset>,
    on_click: EventHandler<Uuid>,
) -> Element {
    rsx! {
        div {
            class: "asset-thumbnail",
            draggable: "true",
            ondragstart: move |e| {
                e.set_data("text/plain", asset.id.to_string());
                on_drag_start.call(asset.clone());
            },
            onclick: move |_| on_click.call(asset.id),
            div {
                class: "thumbnail-container",
                if let Some(thumb_data) = &asset.thumbnail {
                    img {
                        src: format!("data:image/png;base64,{}", base64::encode(thumb_data)),
                        width: "100",
                        height: "60",
                    }
                } else {
                    div {
                        class: "placeholder-thumb",
                        "{asset.asset_type:?}"
                    }
                }
            }
            div {
                class: "asset-name",
                "{asset.name}"
            }
        }
    }
}

#[component]
pub fn ClipComponent(
    clip: Clip,
    track_height: f32,
    timeline_width: f32,
    is_selected: bool,
    on_click: EventHandler<Uuid>,
    on_transform: EventHandler<(Uuid, Transform)>,
) -> Element {
    let width = (clip.duration * 100.0) as f32;
    let left = (clip.start_time * 100.0) as f32;

    let mut local_transform = use_signal(|| clip.transform.clone());

    rsx! {
        div {
            class: if is_selected { "clip selected" } else { "clip" },
            style: "width: {width}px; left: {left}px; height: {track_height - 10}px;",
            onclick: move |_| on_click.call(clip.id),
            draggable: "true",
            ondragstart: move |e| {
                e.set_data("text/plain", clip.id.to_string());
            },
            div {
                class: "clip-content",
                "Clip"
            }
        }
    }
}

#[component]
pub fn TimelineTrack(
    track_id: Uuid,
    track_name: String,
    clips: Vec<Clip>,
    selected_clips: Vec<Uuid>,
    height: f32,
    timeline_width: f32,
    on_clip_click: EventHandler<Uuid>,
    on_clip_transform: EventHandler<(Uuid, Transform)>,
) -> Element {
    let track_ref = use_signal(|| None);

    rsx! {
        div {
            class: "timeline-track",
            style: "height: {height}px;",
            ref: track_ref,
            ondragover: move |e| e.prevent_default(),
            ondrop: move |e| {
                e.prevent_default();
                if let Some(data) = e.data() {
                    if let Ok(asset_id) = data.get("text/plain").parse::<Uuid>() {
                        // Handle drop logic
                    }
                }
            },
            div {
                class: "track-header",
                span { "{track_name}" }
                input {
                    r#type: "range",
                    min: "0",
                    max: "200",
                    value: "100",
                    onchange: move |e| {
                        // Volume control for track
                    }
                }
            }
            div {
                class: "track-clips",
                for clip in clips.iter() {
                    ClipComponent {
                        clip: clip.clone(),
                        track_height: height,
                        timeline_width,
                        is_selected: selected_clips.contains(&clip.id),
                        on_click: on_clip_click,
                        on_transform: on_clip_transform,
                    }
                }
            }
        }
    }
}