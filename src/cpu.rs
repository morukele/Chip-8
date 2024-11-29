use std::time::{Duration, Instant};

const MEMORY_SIZE: usize = 4096; // 4 KB of memory
const STACK_SIZE: usize = 16; // Stack can hold 16 addresses
const NUM_REGISTERS: usize = 16; // 16 general-purpose registers
const DISPLAY_WIDTH: usize = 64; // Default display width
const DISPLAY_HEIGHT: usize = 32; // Default pixel height
const FONT_START: usize = 0x050; // Font starts at memory location 0x050
const FONT_SIZE: usize = 80; // 16 characters * 5 bytes per character
const FONTS: [u8; FONT_SIZE] = [
    // Each number is represented as 5 bytes, 4 pixels wide
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const TIMER_FREQUENCY: u64 = 60; // Timer runs at 60 Hz (FPS)
const TIMER_INTERVAL: Duration = Duration::from_secs(1 / TIMER_FREQUENCY); // should be updated 60 times per second to get 60 FPS
const RUN_FREQUENCY: u64 = 700; // 700 Chip-8 instructions per second
const RUN_INTERVAL: Duration = Duration::from_secs(1 / RUN_FREQUENCY); // should run 700 instructions per second
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],                        // 4 KB of memory
    pub display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT], // 64x32 monochrome display
    program_counter: u16,                             // Program counter (PC), 12-bit addressable
    index_register: u16,                              // index register (I), 12-bit addressable
    stack: [u16; STACK_SIZE],                         // Stack for 16-bit addresses
    delay_timer: u8,                                  // 8-bit delay timer
    sound_timer: u8,                                  // 8-bit sound timer
    registers: [u8; NUM_REGISTERS],                   // 16 8-bit general-purpose registers (V0-VF)
    last_timer_update: Instant,                       // parameter to work with timer update
    last_instruction_execution: Instant,              // parameter to control instruction execution
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT], // screen starts black
            program_counter: 0x200, // offset to the default start address (200 in hex)
            index_register: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; NUM_REGISTERS],
            last_timer_update: Instant::now(), // set counter to instance CPU is created
            last_instruction_execution: Instant::now(),
        }
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self::default();

        // Load font data into memory at 0x050
        chip8.memory[FONT_START..FONT_START + FONT_SIZE].copy_from_slice(&FONTS);

        chip8
    }

    /// A function to decrement the times.
    /// If the values of the timer is above zero,
    /// it should be decremented by one 60 times per second
    pub fn decrement_timers(&mut self) {
        let now = Instant::now();
        let elapsed_time = now.duration_since(self.last_timer_update);

        // check if enough time has passed to decrement timer (60Hz)
        if elapsed_time > TIMER_INTERVAL {
            let ticks = elapsed_time.as_secs_f64() * 60.0; // calculate how many ticks.
            let decrement = ticks as u8;

            if self.delay_timer > 0 {
                self.delay_timer = self.delay_timer.saturating_sub(decrement);
            }

            if self.sound_timer > 0 {
                self.sound_timer = self.sound_timer.saturating_sub(decrement);
            }

            // update the mast timer update time to now
            self.last_timer_update = now; // there is a trivial delay here
        }
    }

    /// Fetch the instruction from memory at the current program counter
    pub fn fetch(&self) {}

    /// Decode the instruction to find out what the emulator should do
    pub fn decode(&self) {}

    /// Execute the instruction and do what it tells you
    pub fn execute(&self) {}

    /// A function to Run the Chip-8 CPU
    pub fn run(&mut self) {
        loop {
            // The run speed of the program should be around 700 instructions per seconds
            let now = Instant::now();
            let time_elapsed = now.duration_since(self.last_instruction_execution);

            // check if enough time has passed before running next command.
            if time_elapsed > RUN_INTERVAL {
                // Run execution code here.
            }

            // Set the timer to the new time
            self.last_instruction_execution = now;
        }
    }
}
