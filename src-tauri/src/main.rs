#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use device_query::{DeviceEvents, DeviceState};
use tauri::Manager;
use cpal::traits::{DeviceTrait,HostTrait};
use cpal::{InputDevices,Devices};

#[derive(Clone, serde::Serialize)]
struct Payload {
    key_pressed: String
}

fn main() {
    let all_output_devices: Vec<InputDevices<Devices>> = cpal::available_hosts()
    .iter()
    .map(|host_id| cpal::host_from_id(*host_id).unwrap().input_devices().unwrap())
    .collect();

    let output_devices = all_output_devices.into_iter().next().unwrap();

    println!("Hosts:");
    output_devices.into_iter().for_each(|x| { println!("{}", x.name().unwrap())});

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
        .run(tauri::generate_context!("./dist"))
        .expect("error while running tauri application");
}
