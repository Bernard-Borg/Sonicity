#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod audio;
use audio::AudioPlayer;
use tauri::Manager;
use std::collections::HashMap;

use device_query::{DeviceEvents, DeviceState};
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
        let host = cpal::host_from_id(host_id).expect("Host not found");
        
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

#[tauri::command]
fn play_audio(_handle: tauri::AppHandle, _audio_path: String, host_id: String, device: String) -> () {
    println!("Hi");
    let selected_host_id = cpal::available_hosts().into_iter()
        .find(|x| x.name() == host_id)
        .expect("Host not found");

    let selected_host = cpal::host_from_id(selected_host_id)
        .expect(format!("Device {host_id} host is unavailable").as_str());

    let device = selected_host.input_devices()
        .expect("Unable to list host devices")
        .find(|x| x.name().expect("Error getting device name") == device);

    let output_device: Device;

    if device.is_none() {
        let default_device = selected_host.default_input_device();

        if !default_device.is_none() {
            output_device = default_device.unwrap();
        } else {
            panic!("No input device found");
        }
    } else {
        output_device = device.unwrap();
    }

    let mut audio_player = AudioPlayer::new();
    audio_player.set_playback_device(output_device).expect("An error occurred while switching playback device");
    audio_player.play_audio("C:\\Users\\jokni\\Desktop\\is-wrong.mp3").expect("An error occured while playing audio");

    return ();
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
            list_audio_devices,
            play_audio
        ])
        .run(tauri::generate_context!("./dist"))
        .expect("error while running tauri application");
}
