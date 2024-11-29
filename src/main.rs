use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use chip_8::Display;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context, 10);

    let buffer = [[false; 32]; 64];

    // display loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                  break 'running
                },
                _=> {}
            }
        }

        display.draw(&buffer);
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
