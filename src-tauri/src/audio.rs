extern crate ffmpeg_next as ffmpeg;

use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

use cpal::traits::{HostTrait, StreamTrait};
use cpal::{Device, SupportedStreamConfig};
use rodio::{DeviceTrait, Source};

pub struct AudioPlayer {
    device: Option<cpal::Device>,
    stream_config: Option<SupportedStreamConfig>
}

impl AudioPlayer {
    // Constructor
    pub fn new() -> AudioPlayer {
        ffmpeg::init().unwrap();

        return AudioPlayer { device: None, stream_config: None };
    }

    pub fn set_playback_device(&mut self, host_id: String, device: String) -> Result<(), Box<dyn std::error::Error>> {
        let selected_host_id = cpal::available_hosts().into_iter()
        .find(|x| x.name() == host_id)
        .expect("Host not found");

        let selected_host = cpal::host_from_id(selected_host_id)
            .expect(format!("Device {host_id} host is unavailable").as_str());

        let device = selected_host.output_devices()
            .expect("Unable to list host devices")
            .find(|x| x.name().expect("Error getting device name") == device);

        let output_device: Device;

        if device.is_none() {
            let default_device = selected_host.default_output_device();

            if !default_device.is_none() {
                output_device = default_device.unwrap();
            } else {
                panic!("No input device found");
            }
        } else {
            output_device = device.unwrap();
        }

        // Create an output stream for the audio so we can play it
        // NOTE: If system doesn't support the file's sample rate, the program will panic when we try to play,
        //       so we'll need to resample the audio to a supported config
        let supported_config_range = output_device.supported_output_configs()
            .expect("error querying audio output configs")
            .next()
            .expect("no supported audio config found");

        self.device = Some(output_device);
        self.stream_config = Some(supported_config_range.with_max_sample_rate());

        return Ok(());
    }
    
    pub fn play_audio(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Load the audio file using Rodio's Decoder
        let file = File::open(file_path)?;
        let source = rodio::Decoder::new(BufReader::new(file))?;
        
        // Convert source into an iterator of samples (f32)
        let samples: Vec<f32> = source.convert_samples().collect();

        // Get the device
        let device = self.device.as_mut().expect("No device selected");

        // Get the default output config for the device
        let config = self.stream_config.clone().expect("Missing stream config");
        let config_clone = self.stream_config.clone().expect("Missing stream config");

        // Create a buffer and wrap it in an Arc<Mutex<>> for thread safety
        let buffer = Arc::new(Mutex::new(samples.into_iter()/*.cycle() for repeat mode ?*/));

        // Build the output stream
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = buffer.next().unwrap_or(0.0);
                    }
                },
                |err| eprintln!("Error: {:?}", err),
                Option::None
            )?,
            cpal::SampleFormat::I32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [i32], _info: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer.lock().unwrap();

                    for sample in data.chunks_mut(config_clone.channels() as usize) {
                        for channel_sample in sample.iter_mut() {
                            *channel_sample = (buffer.next().unwrap_or(0.0) * i32::MAX as f32) as i32
                        }
                    }
                },
                |err| eprintln!("Error: {:?}", err),
                Option::None
            )?,
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config.into(),
                move |data: &mut [i16], _info: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = (buffer.next().unwrap_or(0.0) * i16::MAX as f32) as i16;
                    }
                },
                |err| eprintln!("Error: {:?}", err),
                Option::None
            )?,
            cpal::SampleFormat::U16 => device.build_output_stream(
                &config.into(),
                move |data: &mut [u16], _info: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = ((buffer.next().unwrap_or(0.0) + 1.0) * 0.5 * u16::MAX as f32) as u16;
                    }
                },
                |err| eprintln!("Error: {:?}", err),
                Option::None
            )?,
            cpal::SampleFormat::U8 => device.build_output_stream(
                &config.into(),
                move |data: &mut [u8], _info: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = ((buffer.next().unwrap_or(0.0) + 1.0) * 0.5 * u8::MAX as f32) as u8;
                    }
                },
                |err| eprintln!("Error: {:?}", err),
                Option::None
            )?,
            _ => {
                panic!("{}", format!("Sample format {} not supported", config.sample_format()));
            }
        };

        // Start streaming#
        stream.play()?;
        println!("Streaming audio...");

        std::thread::sleep(std::time::Duration::from_secs(1));
        return Ok(());
    }
}