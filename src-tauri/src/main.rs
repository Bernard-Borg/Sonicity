#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

#[derive(Clone, serde::Serialize)]
struct Payload {
    key_pressed: String
}

fn main() {
    tauri::Builder::default()
        .setup(|app|  {
            let handle = app.handle();
            let handle2 = app.handle();

            use device_query::{DeviceEvents, DeviceState};
            let device_state = DeviceState::new();

            println!("{:?}", device_state);

            let _ = device_state.on_key_down(move |key| {
                let _ = handle.emit_all("keypress", Payload { key_pressed: key.to_string() });
            });

            let _ = device_state.on_key_up(move |key| {
                let _ = handle2.emit_all("keyup", Payload { key_pressed: key.to_string() });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
