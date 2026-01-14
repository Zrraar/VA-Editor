#[cfg(target_os = "windows")]
fn setup_windows() {
    use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
use dioxus::prelude::*;
use video_editor::app::VideoEditorApp;

fn main() {
    #[cfg(target_os = "windows")]
    setup_windows();

    // Initialize logging
    env_logger::init();
    
    // Launch the app
    dioxus_desktop::launch_cfg(
        VideoEditorApp,
        dioxus_desktop::Config::new()
            .with_window(
                dioxus_desktop::WindowBuilder::new()
                    .with_title("Video Editor")
                    .with_inner_size(dioxus_desktop::LogicalSize::new(1400, 900))
                    .with_resizable(true)
                    .with_decorations(true)
            )
    );
}