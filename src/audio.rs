use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::sync::{Arc, Mutex};

// Struct defining the beep sound wave
pub struct SquareWave {
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            // Generate a square wave
            self.phase = (self.phase + 0.02) % 1.0;
            *x = if self.phase < 0.5 {
                self.volume
            } else {
                -self.volume
            };
        }
    }
}

pub fn initialize_audio() -> (sdl2::audio::AudioDevice<SquareWave>, Arc<Mutex<bool>>) {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    // Audio spec
    let spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // Mono
        samples: None,     // Default sample size
    };

    // Shared state to control playback
    let is_playing = Arc::new(Mutex::new(false));

    // Create an audio device
    let device = audio_subsystem
        .open_playback(None, &spec, |_| {
            // Initialize the SquareWave generator
            SquareWave {
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();

    (device, is_playing)
}
