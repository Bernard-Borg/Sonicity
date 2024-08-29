#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod audio;
use audio::AudioPlayer;
use once_cell::sync::Lazy;
use tauri::Manager;
use std::collections::HashMap;
use std::sync::Mutex;

use device_query::{DeviceEvents, DeviceState};
use cpal::traits::{DeviceTrait,HostTrait};
use cpal::Device;

static AUDIO_PLAYER: Lazy<Mutex<AudioPlayer>> = Lazy::new(|| Mutex::new(AudioPlayer::new()));

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
fn change_device(_handle: tauri::AppHandle, host_id: String, device: String) -> () {
    AUDIO_PLAYER.lock().unwrap().set_playback_device(host_id, device).expect("An error occurred while switching playback device");
    play_audio(_handle, "C:\\Users\\jokni\\Desktop\\is-wrong.mp3".to_string());

    return ();
}

#[tauri::command]
fn play_audio(_handle: tauri::AppHandle, audio_path: String) -> () {
    AUDIO_PLAYER.lock().unwrap().play_audio(&audio_path).expect("An error occured while playing audio");

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
            play_audio,
            change_device
        ])
        .run(tauri::generate_context!("./dist"))
        .expect("error while running tauri application");
}
