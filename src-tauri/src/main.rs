#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

#[derive(Clone, serde::Serialize)]
struct Payload {
    key_pressed: String
}

#[tauri::command]
fn listen_to_keys(app_handle: tauri::AppHandle) {
    use device_query::{DeviceEvents, DeviceState};
    let device_state = DeviceState::new();
    
    println!("Hi");

    let _guard = device_state.on_key_down(move |key| {
        println!("Keyboard key down: {:?}", key);
        app_handle.emit_all("keypress", Payload { key_pressed: key.to_string() });
    });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![listen_to_keys])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
