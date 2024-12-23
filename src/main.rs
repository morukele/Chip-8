use chip_8::{initialize_audio, Chip8, Display};
use clap::{Arg, ArgMatches, Command};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;
use std::time::{Duration, Instant};

const RUN_FREQUENCY: u64 = 700; // 700 Chip-8 instructions per second
const RUN_INTERVAL: Duration = Duration::from_micros(1_000_000 / RUN_FREQUENCY); // should cycle 700 instructions per second

fn main() {
    // Getting CLI info
    let matches = Command::new("CHIP-8 emulator")
        .version("0.1")
        .author("Oghenemarho ORUKELE <orukele.dev@gmail.com>")
        .about("A CHIP-8 emulator written in Rust")
        .arg(
            Arg::new("ROM")
                .short('r')
                .long("rom")
                .help("Name of CHIP-8 ROM file (remember to specify the extension)")
                .required(true)
                .default_value("1-chip8-logo"),
        )
        .arg(
            Arg::new("scale")
                .short('s')
                .help("Set the scale of the display")
                .required(false)
                .default_value("10")
                .default_missing_value("10"),
        )
        .get_matches();

    // Extract arguments
    let (rom_name, scale) = extract_arguments(matches);

    // Run emulator
    run_emulator(&rom_name, scale);
}

fn extract_arguments(matches: ArgMatches) -> (String, u32) {
    let rom_name = matches
        .get_one::<String>("ROM")
        .expect("unable to get ROM name")
        .to_owned();
    let scale: u32 = matches
        .get_one::<String>("scale")
        .expect("unable to get scale factor")
        .parse()
        .unwrap_or(10);
    (rom_name, scale)
}

fn run_emulator(rom_name: &String, scale: u32) {
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context, scale);

    let rom_path = format!("./rom/{}", rom_name);
    let path = Path::new(&rom_path);
    let rom = std::fs::read(path).expect("Unable to read file");

    let mut chip8 = Chip8::new(false); // create new instance of Chip-8
    chip8.load_rom(rom); // load rom

    let (audio_device, is_playing) = initialize_audio(); // initialize audio with SDL2

    let mut start = Instant::now(); // set up timer to ensure run of 700 instruction per second

    // main loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(chip8_key) = map_key(key) {
                        chip8.keypad[chip8_key] = true; // Set key pressed to true
                    }

                    // Check escape key
                    if key == Keycode::ESCAPE {
                        break 'running;
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(chip8_key) = map_key(key) {
                        chip8.keypad[chip8_key] = false; // Set key unpressed to false
                    }
                }
                Event::Quit { .. } => {
                    std::process::exit(0); // Exit on quit event
                }
                _ => {}
            }
        }

        let elapsed_time = start.elapsed(); // get the time elapsed
        if elapsed_time >= RUN_INTERVAL {
            // check if elapsed time is greater than run interval
            chip8.cycle(); // chip 8 cycle here
            display.draw(&chip8.display); // render the CHIP-8 display
            chip8.update_sound(&audio_device, &is_playing);
            chip8.update_timers(); // update timers
            start = Instant::now(); // update the run timer to now
        } else {
            // This is to prevent Busy-Wait loop.
            std::thread::sleep(RUN_INTERVAL - elapsed_time);
        }
    }
}

fn map_key(key: Keycode) -> Option<usize> {
    // Map host keyboard keys to CHIP-8 keys
    // NOTE: This works for 'AZERTY' keyboard only
    match key {
        Keycode::NUM_1 => Some(0x1),
        Keycode::NUM_2 => Some(0x2),
        Keycode::NUM_3 => Some(0x3),
        Keycode::NUM_4 => Some(0xC),
        Keycode::A => Some(0x4),
        Keycode::Z => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::Q => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::W => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None, // Ignore other keys
    }
}
