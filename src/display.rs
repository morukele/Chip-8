extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

const DISPLAY_WIDTH: u32 = 64; // Default display width
const DISPLAY_HEIGHT: u32 = 32; // Default pixel height
pub struct Display {
    canvas: Canvas<Window>,
    scale: u32,
    background_color: Color,
    foreground_color: Color,
}

impl Display {
    pub fn new(sdl_context: &Sdl, scale: u32) -> Self {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Chip-8", DISPLAY_WIDTH * scale, DISPLAY_HEIGHT * scale)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        Self {
            canvas,
            scale,
            background_color: Color::RGB(0, 0, 0),
            foreground_color: Color::RGB(255, 255, 255),
        }
    }

    pub fn draw(
        self: &mut Display,
        buffer: &[[bool; DISPLAY_WIDTH as usize]; DISPLAY_HEIGHT as usize],
    ) {
        self.canvas.set_draw_color(self.background_color);
        self.canvas.clear();

        self.canvas.set_draw_color(self.foreground_color);
        for (r, row) in buffer.iter().enumerate() {
            for (c, col) in row.iter().enumerate() {
                if *col {
                    let r = ((r as u32) * self.scale) as i32;
                    let c = ((c as u32) * self.scale) as i32;
                    let width = self.scale;
                    let height = self.scale;
                    self.canvas
                        .fill_rect(Rect::new(r, c, width, height))
                        .expect("Failed to draw pixel");
                }
            }
        }

        self.canvas.present();
    }
}
