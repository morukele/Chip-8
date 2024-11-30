use chip_8::{Chip8, Display};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;
use std::time::{Duration, Instant};

const RUN_FREQUENCY: u64 = 700; // 700 Chip-8 instructions per second
const RUN_INTERVAL: Duration = Duration::from_micros(1_000_000 / RUN_FREQUENCY); // should cycle 700 instructions per second

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context, 10);

    let path = Path::new("./rom/Life [GV Samways, 1980].ch8");
    let rom = std::fs::read(path).expect("Unable to read file");

    let mut chip8 = Chip8::new();       // create new instance of Chip-8
    chip8.load_rom(rom);                       // load rom
    let mut start = Instant::now();    // set up timer to ensure run of 700 instruction per second

    // main loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let elapsed_time = start.elapsed(); // get the time elapsed
        if elapsed_time > RUN_INTERVAL {             // check if elapsed time is greater than run interval
            chip8.cycle();                           // chip 8 cycle here
            display.draw(&chip8.display);            // render the CHIP-8 display
            chip8.decrement_timers();                // update timers
            start = Instant::now();                  // update the run timer to now
        }
    }
}
