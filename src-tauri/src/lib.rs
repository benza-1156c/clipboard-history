// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, Rgba};
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::{hash::DefaultHasher, thread, time::Duration};
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Clone)]
pub struct ClipboardPayload {
    pub kind: String,
    pub content: String,
}

pub fn start_clipboard_watcher(app: AppHandle) {
    thread::spawn(move || {
        let mut clipboard = Clipboard::new().unwrap();
        let mut last = String::new();
        let mut last_image_hash: Option<u64> = None;

        loop {
            if let Ok(text) = clipboard.get_text() {
                if text != last {
                    last = text.clone();

                    let payload = ClipboardPayload {
                        kind: "text".into(),
                        content: text,
                    };

                    let _ = app.emit("clipboard-changed", payload);
                }
            }

            if let Ok(image) = clipboard.get_image() {
                let hash = calculate_image_hash(&image);
                if Some(hash) != last_image_hash {
                    last_image_hash = Some(hash);

                    let width = image.width as u32;
                    let height = image.height as u32;
                    let image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
                        ImageBuffer::from_raw(width, height, image.bytes.to_vec()).unwrap();

                    let mut buffer = Cursor::new(Vec::new());
                    image_buffer
                        .write_to(&mut buffer, image::ImageFormat::Png)
                        .unwrap();

                    let base64 = general_purpose::STANDARD.encode(buffer.get_ref());
                    let payload = ClipboardPayload {
                        kind: "image".into(),
                        content: base64,
                    };

                    let _ = app.emit("clipboard-changed", payload);
                }
            }

            thread::sleep(Duration::from_millis(300));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            start_clipboard_watcher(app_handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn calculate_image_hash(image: &ImageData) -> u64 {
    let mut hasher = DefaultHasher::new();
    image.bytes.hash(&mut hasher);
    hasher.finish()
}
