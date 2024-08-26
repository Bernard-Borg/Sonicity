extern crate ffmpeg_next as ffmpeg;

use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

use cpal::traits::{HostTrait, StreamTrait};
use cpal::{Device, SupportedStreamConfig};
use rodio::{DeviceTrait, Source};

pub struct AudioPlayer {
    device: cpal::Device,
    stream_config: SupportedStreamConfig
}   

impl AudioPlayer {
    // Constructor
    pub fn new() -> AudioPlayer {
        ffmpeg::init().unwrap();

        // Initialize cpal for playing audio
        let (device, stream_config) = Self::init_cpal();

        return AudioPlayer { device: device, stream_config: stream_config };
    }

    pub fn set_playback_device(&mut self, device: Device) -> Result<(), Box<dyn std::error::Error>> {
        self.device = device;
        return Ok(());
    }
    
    pub fn play_audio(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Load the audio file using Rodio's Decoder
        let file = File::open(file_path)?;
        let source = rodio::Decoder::new(BufReader::new(file))?;

        // Convert source into an iterator of samples (f32)
        let samples: Vec<f32> = source.convert_samples().collect();

        // Get the device
        let device = &self.device;

        // Get the default output config for the device
        let config = self.stream_config.clone();

        // Create a buffer and wrap it in an Arc<Mutex<>> for thread safety
        let buffer = Arc::new(Mutex::new(samples.into_iter().cycle()));

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
            _ => {
                panic!("Sample format not supported");
            }
        };

        // Start streaming
        stream.play()?;
        println!("Streaming audio...");

        return Ok(());
    }

    
    fn init_cpal() -> (cpal::Device, cpal::SupportedStreamConfig) {
        let device = cpal::default_host()
            .default_output_device()
            .expect("no output device available");
    
        // Create an output stream for the audio so we can play it
        // NOTE: If system doesn't support the file's sample rate, the program will panic when we try to play,
        //       so we'll need to resample the audio to a supported config
        let supported_config_range = device.supported_output_configs()
            .expect("error querying audio output configs")
            .next()
            .expect("no supported audio config found");
    
        // Pick the best (highest) sample rate
        (device, supported_config_range.with_max_sample_rate())
    }
    
}