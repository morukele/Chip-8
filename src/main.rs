use chip_8::{Chip8, Display};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;
use std::time::Duration;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context, 10);

    let path = Path::new("./rom/IBM Logo.ch8");
    let rom = std::fs::read(path).expect("Unable to read file");

    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);

    // display loop
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

        // chip 8 cycle here
        chip8.cycle();

        // render the CHIP-8 display
        display.draw(&chip8.display);

        // update timers
        chip8.decrement_timers();
    }
}
