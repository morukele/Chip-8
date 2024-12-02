use crate::SquareWave;
use rand::Rng;
use sdl2::audio::AudioDevice;
use std::sync::{Arc, Mutex};
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
const PROGRAM_START: usize = 0x200;
const TIMER_FREQUENCY: u64 = 60; // Timer runs at 60 Hz (FPS)
const TIMER_INTERVAL: Duration = Duration::from_micros(1_000_000 / TIMER_FREQUENCY); // should be updated 60 times per second to get 60 FPS
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE], // 4 KB of memory
    // NB: the dimensioning is w*h; width represents the columns, and height represents the rows
    // This is a bit confusing for now.
    pub display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT], // 64x32 monochrome display
    program_counter: u16,           // Program counter (PC), 12-bit addressable
    index_register: u16,            // index register (I), 12-bit addressable
    stack: [u16; STACK_SIZE],       // Stack for 16-bit addresses
    delay_timer: u8,                // 8-bit delay timer
    sound_timer: u8,                // 8-bit sound timer
    registers: [u8; NUM_REGISTERS], // 16 8-bit general-purpose registers (V0-VF)
    last_timer_update: Instant,     // parameter to work with timer update
    stack_pointer: usize,           // parameter for tracking the position on the stack during calls
    pub keypad: [bool; 16],         // bool array to hold the key information
    modern: bool,                   // bool to determine if to use modern implementation or not
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT], // screen starts black
            program_counter: PROGRAM_START as u16, // offset to the default start address (200 in hex)
            index_register: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; NUM_REGISTERS],
            last_timer_update: Instant::now(), // set counter to instance CPU is created
            stack_pointer: 0,                  // stack starts at zero
            keypad: [false; 16],               // all keys start as unpressed
            modern: false,                     // determine if the modern implementation is used
        }
    }
}

impl Chip8 {
    pub fn new(modern: bool) -> Self {
        let mut chip8 = Chip8 {
            modern,
            ..Default::default()
        };
        chip8.modern = modern;

        // Load font data into memory at 0x050
        for (i, font) in FONTS.iter().enumerate() {
            chip8.memory[FONT_START + i] = *font;
        }

        chip8
    }

    /// A function to load the ROM into memory
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.memory[self.program_counter as usize + i] = *byte
        }
    }

    /// A function to decrement the times.
    /// If the values of the timer is above zero,
    /// it should be decremented by one 60 times per second
    pub fn update_timers(&mut self) {
        let elapsed_time = self.last_timer_update.elapsed();

        // check if enough time has passed to decrement timer (60Hz)
        if elapsed_time > TIMER_INTERVAL {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }

            // update the mast timer update time to now
            self.last_timer_update = Instant::now(); // there is a trivial delay here
        }
    }

    pub fn update_sound(
        &mut self,
        audio_device: &AudioDevice<SquareWave>,
        is_playing: &Arc<Mutex<bool>>,
    ) {
        if self.sound_timer > 0 {
            // Start playing sound if not already playing
            let mut playing = is_playing.lock().unwrap();
            if !*playing {
                audio_device.resume();
                *playing = true;
            }
        } else {
            // Stop playing sound if timer reaches 0
            let mut playing = is_playing.lock().unwrap();
            if *playing {
                audio_device.pause();
                *playing = false;
            }
        }
    }

    /// Fetch the instruction from memory at the current program counter
    pub fn fetch(&mut self) -> u16 {
        // An instruction is two successive bytes that is combined to 16-bit instruction
        let pc = self.program_counter as usize;
        let op_byte1 = self.memory[pc] as u16;
        let op_byte2 = self.memory[pc + 1] as u16;

        // increment program counter by 2
        self.program_counter += 2;

        // combine the two bytes into a single 16 bit output
        op_byte1 << 8 | op_byte2
    }

    /// Decode the instruction to find out what the emulator should do
    pub fn decode(&mut self, &opcode: &u16) -> (u8, u8, u8, u8, u8, u16) {
        // Opcode decoding
        // the first `nibble` is the category of instruction
        // Mask off using a binary AND, then shift 12 places and truncate to u8.
        let c = ((opcode & 0xF000) >> 12) as u8; // first nibble (operation category)
        let x = ((opcode & 0x0F00) >> 8) as u8; // second nibble (register loop up)
        let y = ((opcode & 0x00F0) >> 4) as u8; // third nibble (register look up)
        let n = (opcode & 0x000F) as u8; // fourth nibble (a 4-bit number)
        let nn = (opcode & 0x00FF) as u8; // second byte (an 8-bit immediate number)
        let nnn = opcode & 0x0FFF; // second, third and fourth nibbles (a 12 bit immediate memory address)

        (c, x, y, n, nn, nnn)
    }

    /// A function to Run the Chip-8 CPU
    pub fn cycle(&mut self) {
        // Run emulator here.
        // get and decode opcode
        let opcode = self.fetch();
        let (c, x, y, n, nn, nnn) = self.decode(&opcode);

        let vx = self.registers[x as usize]; // value at x in the register
        let vy = self.registers[y as usize]; // value at y in the register

        // matching the operation category first
        // TODO: clean up matching, especially where parameters are discarded.
        match c {
            0x0 => {
                // operations in case 0x0
                match (x, y, n) {
                    (0, 0, 0) => {}
                    (0, 0xE, 0) => {
                        // 0x00E0: Clear screen
                        println!("Handling opcode: {:#x?} - clearing display", opcode);
                        self.display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
                    }
                    (0, 0xE, 0xE) => {
                        // 0x00EE: return subroutine
                        println!("Handling opcode: {:#x?} - return subroutine", opcode);
                        self.return_subroutine();
                    }
                    _ => panic!("Unimplemented opcode: {:#x?}", opcode),
                }
            }
            0x1 => {
                // 0x1NNN: Jump to NNN address
                println!(
                    "Handling opcode: {:#x?} - setting program counter to {}",
                    opcode, nnn
                );
                self.program_counter = nnn;
            }
            0x2 => {
                // 0x2NNN: call_subroutine subroutine at nnn
                println!(
                    "Handling opcode: {:#x?} - call subroutine at {:#x?}",
                    opcode, nnn
                );
                self.call_subroutine(nnn);
            }
            0x3 => {
                // 0x3XNN: skip conditionally
                println!(
                    "Handling opcode: {:#x?} - skip one if VX({}) == NN({})",
                    opcode, vx, nn
                );
                if vx == nn {
                    self.program_counter += 2;
                }
            }
            0x4 => {
                // 0x4XNN: skip conditionally
                println!(
                    "Handling opcode: {:#x?} - skip one if VX({}) != NN({})",
                    opcode, vx, nn
                );
                if vx != nn {
                    self.program_counter += 2;
                }
            }
            0x5 => {
                // 0x5XY0: skip conditionally
                println!(
                    "Handling opcode: {:#x?} - skip one if VX({}) == VY({})",
                    opcode, vx, vy
                );
                if vx == vy {
                    self.program_counter += 2;
                }
            }
            0x6 => {
                // 6XNN: Set VX to NN
                println!(
                    "Handling opcode: {:#x?} - setting v{} register to {}",
                    opcode, x, nn
                );
                self.registers[x as usize] = nn;
            }
            0x7 => {
                // 7XNN: Add value to register VX
                println!(
                    "Handling opcode: {:#x?} - adding {} to v{} register",
                    opcode, nn, x
                );
                self.registers[x as usize] = self.registers[x as usize].wrapping_add(nn);
            }
            0x8 => {
                match n {
                    0x0 => {
                        // 0x8XY0: Set
                        println!(
                            "Handling opcode: {:#x?} - setting v{} to v{}",
                            opcode, vx, vy
                        );
                        self.registers[x as usize] = self.registers[y as usize];
                    }
                    0x1 => {
                        // 0x8XY1: Binary OR
                        println!("Handling opcode: {:#x?} - setting  v{} to binary OR of v{} and v{} register", opcode, x, x, y);
                        self.registers[x as usize] = vx | vy;
                    }
                    0x2 => {
                        // 0x8XY2: Binary AND
                        println!("Handling opcode: {:#x?} - setting  v{} to binary AND of v{} and v{} register", opcode, x, x, y);
                        self.registers[x as usize] = vx & vy;
                    }
                    0x3 => {
                        // 0x8XY3: Logical XOR
                        println!("Handling opcode: {:#x?} - setting  v{} to logical XOR of v{} and v{} register", opcode, x, x, y);
                        self.registers[x as usize] = vx ^ vy;
                    }
                    0x4 => {
                        // 0x8XY4: Add overflowing
                        println!("Handling opcode: {:#x?} - setting v{} to the sum of v{} and v{} register", opcode, x, x, y);
                        self.add_xy(x, y);
                    }
                    0x5 => {
                        // 0x8XY5: VX - VY
                        println!("Handling opcode: {:#x?} - setting v{} to the diff of v{} and v{} register", opcode, x, x, y);
                        self.subtract_xy(x, y);
                    }
                    0x7 => {
                        // 0x8XY5: VY - VX
                        println!("Handling opcode: {:#x?} - setting v{} to the diff of v{} and v{} register", opcode, x, y, x);
                        self.subtract_yx(x, y);
                    }
                    0x6 => {
                        // 0x8XY6: Shift Right
                        println!("Handling opcode: {:#x?} - shifting v{} >> 1", opcode, x);
                        if !self.modern {
                            // set VX to the value of VY
                            self.registers[x as usize] = self.registers[y as usize]
                            // Set VX to the value of VY
                        }
                        let vx_pre_shift = self.registers[x as usize]; // value of vx before the shift operation
                        self.registers[x as usize] >>= 1; // Shift VX one bit to the right

                        self.registers[0xF] = if vx_pre_shift & 0b0000_0001 != 0 {
                            1
                        } else {
                            0
                        };
                        // set register values
                    }
                    0xE => {
                        // 0x8XYE: Shift Left
                        println!("Handling opcode: {:#x?} - shifting v{} << 1", opcode, x);
                        if !self.modern {
                            // set VX to the value of VY
                            self.registers[x as usize] = self.registers[y as usize];
                            // Set VX to the value of VY
                        }
                        let vx_pre_shift = self.registers[x as usize]; // value of vx before the shift operation
                        self.registers[x as usize] <<= 1; // Shift VX one bit to the left

                        self.registers[0xF] = if vx_pre_shift & 0b1000_0000 != 0 {
                            1
                        } else {
                            0
                        }; // set register values
                    }
                    _ => panic!("Unimplemented opcode: {:#x?}", opcode),
                }
            }
            0x9 => {
                // 0x9XY0: skip conditionally
                println!(
                    "Handling opcode: {:#x?} - skip one if VX({}) =! VY({})",
                    opcode, vx, vy
                );
                if vx != vy {
                    self.program_counter += 2;
                }
            }
            0xA => {
                // ANNN: Set index register I to NNN
                println!(
                    "Handling opcode: {:#x?} - setting index register to {}",
                    opcode, nnn
                );
                self.index_register = nnn;
            }
            0xB => {
                // 0xBNNN: Jump with offset
                // TODO: add support for "qurik" configuration
                println!(
                    "Handling opcode: {:#x?} - jump to address {} + {}",
                    opcode, nnn, self.registers[0]
                );
                self.program_counter = nnn + self.registers[0] as u16;
            }
            0xC => {
                // OxCXNN: Random
                let rand_num: u8 = rand::rng().random();
                self.registers[x as usize] = nn & rand_num;
            }
            0xD => {
                // DXYN: draw
                println!(
                    "Handling opcode: {:#x?}. drawing sprite of {} rows at ({}, {})",
                    opcode, n, x, y
                );
                // N = height of the sprite
                // X = horizontal coordinate in VX
                // Y = vertical coordinate in VY
                let x_start = vx % DISPLAY_WIDTH as u8; // X coordinate
                let y_start = vy % DISPLAY_HEIGHT as u8; // Y coordinate
                self.registers[0xF] = 0; // Set VF to 0

                for row in 0..n {
                    let y = y_start + row;
                    if y >= DISPLAY_HEIGHT as u8 {
                        break;
                    }
                    let sprite = self.memory[self.index_register as usize + row as usize];

                    for col in 0..8 {
                        // Check if the bite for the column is set
                        let x = x_start + col;
                        if x >= DISPLAY_WIDTH as u8 {
                            break;
                        }
                        let on = (sprite >> (7 - col)) & 1 == 1;
                        if on {
                            if self.display[y as usize][x as usize] {
                                self.registers[0xF] = 1; // sprite was active
                            }

                            // Toggle the pixel
                            // exclusive OR will only produce true if the two values are different
                            // i.e. true ^ true = false and true ^ false = true
                            self.display[y as usize][x as usize] ^= true;
                        }
                    }
                }
            }
            0xE => {
                match (y, n) {
                    (0x9, 0xE) => {
                        // 0xEX9E: Skip if key == vx pressed
                        println!(
                            "Handling opcode: {:#x?} - skipping if key pressed == v{}",
                            opcode, x
                        );
                        let key = self.registers[x as usize] as usize; // Key value from VX
                        if key < 16 && self.keypad[key] {
                            // use the less than 16 guard to prevent overflow crashing
                            self.program_counter += 2;
                        }
                    }
                    (0xA, 0x1) => {
                        // 0xEXA1: Skip if key == vx not pressed
                        println!(
                            "Handling opcode: {:#x?} - skipping if key pressed != v{}",
                            opcode, x
                        );
                        let key = self.registers[x as usize] as usize;
                        if key < 16 && !self.keypad[key] {
                            self.program_counter += 2;
                        }
                    }
                    _ => panic!("Unimplemented opcode: {:#x?}", opcode),
                }
            }
            0xF => {
                // Timer code
                match (y, n) {
                    (0x0, 0x7) => {
                        // 0xFX07: sets VX to the current value of the delay timer
                        println!(
                            "Handling opcode: {:#x?} - setting v{} to {}",
                            opcode, x, self.delay_timer
                        );
                        self.registers[x as usize] = self.delay_timer;
                    }
                    (0x1, 0x5) => {
                        // 0xFX15: set the delay timer to the value in VX
                        println!(
                            "Handling opcode: {:#x?} - setting delayer timer to v{}",
                            opcode, x
                        );
                        self.delay_timer = self.registers[x as usize];
                    }
                    (0x1, 0x8) => {
                        // 0xFX18: set the sound timer to the value of VX
                        println!(
                            "Handling opcode: {:#x?} - setting sound timer to v{}",
                            opcode, x
                        );
                        self.sound_timer = self.registers[x as usize];
                    }
                    (0x1, 0xE) => {
                        // 0xFX1E: Add to index
                        println!(
                            "Handling opcode: {:#x?} - adding value of v{} to index register",
                            opcode, x
                        );
                        let (val, overflow) = self.index_register.overflowing_add(vx as u16);
                        self.index_register = val;
                        self.registers[0xF] = if overflow { 1 } else { 0 }; // doing this because of some issues.
                    }
                    (0x0, 0xA) => {
                        // 0xFX0A: Get Key
                        println!("Handling opcode: {:#x?} - Getting Key", opcode);

                        let mut wait = true; // indicate if wait is needed.
                                             // check if key is pressed
                        for (key, pressed) in self.keypad.iter().enumerate() {
                            if *pressed {
                                self.registers[x as usize] = key as u8;
                                wait = false; // if key is pressed, exit wait state
                                self.keypad[key] = false;
                                break;
                            }
                        }

                        // wait until a key is pressed
                        if wait {
                            self.program_counter -= 2;
                        }
                    }
                    (0x2, 0x9) => {
                        // OxFX29: Font Character
                        println!(
                            "Handling opcode: {:#x?} - setting index register to font at v{}",
                            opcode, x
                        );
                        let character = vx & 0xF; // Get the last nibble of VX and set it as character
                        self.index_register = FONT_START as u16 + (0x5 * character) as u16
                        // multiply by 0x5 because each character is represented by 5 bytes
                    }
                    (0x3, 0x3) => {
                        // 0xFX33: Binary-coded decimal conversion
                        // vx = a number from 0 to 255
                        println!(
                            "Handling opcode: {:#x?} - converting v{} to decimal",
                            opcode, x
                        );
                        let hundreds = vx / 100; // will give the value at 100 and truncate remainders
                        let tens = (vx % 100) / 10; // get the remainder by eliminating the 100 digit and divide by 10
                        let units = vx % 10; // get the remainder by modulo 10

                        self.memory[self.index_register as usize] = hundreds;
                        self.memory[self.index_register as usize + 1] = tens;
                        self.memory[self.index_register as usize + 2] = units;
                    }
                    (0x5, 0x5) => {
                        // 0xFX55: store register value from 0..X into memory
                        println!(
                            "Handling opcode: {:#x?} - copying {} values from registers",
                            opcode, x
                        );
                        // TODO: configure for backwards compatability
                        for i in 0..=x {
                            self.memory[(self.index_register + i as u16) as usize] =
                                self.registers[i as usize];
                            println!(
                                "Ram location is at: {} with value: {}",
                                self.index_register + i as u16,
                                self.memory[(self.index_register + i as u16) as usize]
                            );
                        }
                    }
                    (0x6, 0x5) => {
                        // 0xF65:
                        // TODO: configure for backwards compatability
                        println!(
                            "Handling opcode: {:#x?} - copying {} values to registers",
                            opcode, x
                        );
                        for i in 0..=x {
                            self.registers[i as usize] =
                                self.memory[(self.index_register + i as u16) as usize];
                            println!(
                                "Register location is at: {} with value: {}",
                                i, self.registers[i as usize]
                            );
                        }
                    }
                    _ => panic!("Unimplemented opcode: {:#x?}", opcode),
                }
            }
            _ => panic!("Unimplemented opcode: {:#x?}", opcode),
        }
    }

    /// Function to call_subroutine subroutine at address location
    fn call_subroutine(&mut self, addr: u16) {
        // Guard to prevent stack overflow
        if self.stack_pointer >= self.stack.len() {
            panic!("Stack overflow!")
        }
        self.stack[self.stack_pointer] = self.program_counter; // pushing into the current stack location
        self.stack_pointer += 1;
        self.program_counter = addr; // set program counter to the nnn address
    }

    /// Function to return the subroutine and setting the address
    fn return_subroutine(&mut self) {
        // Guard to prevent stack underflow
        if self.stack_pointer == 0 {
            panic!("Stack underflow!")
        }
        self.stack_pointer -= 1;
        let addr = self.stack[self.stack_pointer];
        self.program_counter = addr;
    }

    /// Function adding x and y values while setting the reminder bit
    fn add_xy(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let (val, overflow) = vx.overflowing_add(vy);
        self.registers[x as usize] = val;

        // set the overflow register
        self.registers[0xF] = if overflow { 1 } else { 0 };
    }

    /// Function subtracting x and y values in the register while setting the reminder bit
    fn subtract_xy(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let val = vx.wrapping_sub(vy);
        self.registers[x as usize] = val;

        // setting the overflow register
        self.registers[0xF] = if vx > vy { 1 } else { 0 };
    }

    /// Function subtracting y and x values in the register while setting the reminder bit
    fn subtract_yx(&mut self, x: u8, y: u8) {
        let vx = self.registers[x as usize];
        let vy = self.registers[y as usize];

        let val = vy.wrapping_sub(vx);
        self.registers[x as usize] = val;

        // setting the overflow register
        self.registers[0xF] = if vy > vx { 1 } else { 0 };
    }
}
