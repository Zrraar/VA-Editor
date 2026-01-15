use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            style: "
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                padding: 40px;
                text-align: center;
            ",
            
            h1 {
                style: "
                    color: #2c3e50;
                    font-size: 3em;
                    margin-bottom: 20px;
                ",
                "🎬 Video Editor"
            }
            
            p {
                style: "
                    color: #7f8c8d;
                    font-size: 1.2em;
                    margin-bottom: 40px;
                ",
                "Built with Rust and Dioxus 0.7.2"
            }
            
            div {
                style: "
                    display: flex;
                    gap: 20px;
                    justify-content: center;
                    flex-wrap: wrap;
                ",
                
                button {
                    style: "
                        background: #3498db;
                        color: white;
                        border: none;
                        padding: 15px 30px;
                        border-radius: 10px;
                        font-size: 1.1em;
                        cursor: pointer;
                        transition: background 0.3s;
                    ",
                    onmouseenter: |_| log::info!("Button hover"),
                    onclick: |_| log::info!("Import clicked"),
                    "📁 Import Media"
                }
                
                button {
                    style: "
                        background: #2ecc71;
                        color: white;
                        border: none;
                        padding: 15px 30px;
                        border-radius: 10px;
                        font-size: 1.1em;
                        cursor: pointer;
                        transition: background 0.3s;
                    ",
                    onclick: |_| log::info!("New Project clicked"),
                    "✨ New Project"
                }
                
                button {
                    style: "
                        background: #e74c3c;
                        color: white;
                        border: none;
                        padding: 15px 30px;
                        border-radius: 10px;
                        font-size: 1.1em;
                        cursor: pointer;
                        transition: background 0.3s;
                    ",
                    onclick: |_| log::info!("Export clicked"),
                    "📤 Export Video"
                }
            }
            
            div {
                style: "
                    margin-top: 50px;
                    padding: 20px;
                    background: #ecf0f1;
                    border-radius: 10px;
                    max-width: 800px;
                    margin-left: auto;
                    margin-right: auto;
                ",
                
                h2 { "Features" }
                ul {
                    li { "Video timeline with multiple tracks" }
                    li { "Drag and drop media import" }
                    li { "Real-time preview" }
                    li { "Export to multiple formats" }
                    li { "Built with Rust for performance" }
                }
            }
        }
    }
}