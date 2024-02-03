#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::HashMap;

use device_query::{DeviceEvents, DeviceState};
use tauri::Manager;
use cpal::traits::{DeviceTrait,HostTrait};
use cpal::Device;

#[derive(Clone, serde::Serialize)]
struct Payload {
    key_pressed: String
}

#[tauri::command]
fn list_audio_devices(_handle: tauri::AppHandle, input: bool) -> (HashMap<String, Vec<String>>, String) {
    let mut devices = HashMap::new();
    let hosts = cpal::available_hosts().into_iter();

    _ = hosts.map(|host_id| {
        let hostname = host_id.name().to_string();
        let host = cpal::host_from_id(host_id).expect("No hosts found");
        
        let host_devices;

        if input {
            host_devices = host.input_devices()
                .unwrap()
                .map(|x: Device| x.name().unwrap())
                .collect();
        } else {
            host_devices = host.output_devices()
                .unwrap()
                .map(|x: Device| x.name().unwrap())
                .collect();
        }

        devices.insert(hostname, host_devices);
    }).collect::<Vec<_>>();

    return (devices, cpal::default_host().id().name().to_string());
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            let app_handle2 = app.app_handle();

            tauri::async_runtime::spawn(async {
                let device_state = DeviceState::new();

                let _guard = device_state.on_key_down( move |key| {
                    let _ = app_handle.emit_all("keypress", Payload { key_pressed: key.to_string() });
                });

                let _guard2 = device_state.on_key_up( move |key| {
                    let _ = app_handle2.emit_all("keyup", Payload { key_pressed: key.to_string() });
                });

                loop { }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_audio_devices
        ])
        .run(tauri::generate_context!("./dist"))
        .expect("error while running tauri application");
}
