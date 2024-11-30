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
const RUN_FREQUENCY: u64 = 700; // 700 Chip-8 instructions per second
const RUN_INTERVAL: Duration = Duration::from_secs(1 / RUN_FREQUENCY); // should cycle 700 instructions per second
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
    last_instruction_execution: Instant, // parameter to control instruction execution
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
            last_instruction_execution: Instant::now(),
        }
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self::default();

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
    pub fn decrement_timers(&mut self) {
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

        // matching the operation category first
        match (c, x, y, n) {
            (0, 0, 0, 0) => {

            },
            (0x0, _, _, _) => {
                // operations in case 0x0
                match (x, y, n) {
                    (0, 0xE, 0) => {
                        // Clear the screen
                        println!("Handling opcode: {:#x?} - clearing display", opcode);
                        self.display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
                    }
                    _ => panic!("Unimplemented opcode: {:#x?}", opcode),
                }
            }
            (0x1, _, _, _) => {
                // operation in case 0x1: Jump to NNN address
                println!("Handling opcode: {:#x?} - setting program counter to {}", opcode, nnn);
                self.program_counter = nnn;
            }
            (0x2, _, _, _) => {}
            (0x3, _, _, _) => {}
            (0x4, _, _, _) => {}
            (0x5, _, _, _) => {}
            (0x6, _, _, _) => {
                // 6XNN: Set VX to NN
                println!("Handling opcode: {:#x?} - setting v{} register to {}", opcode, x, nn);
                self.registers[x as usize] = nn;
            }
            (0x7, _, _, _) => {
                // 7XNN: Add value to register VX
                println!("Handling opcode: {:#x?} - adding {} to v{} register", opcode, nn, x);
                self.registers[x as usize] = self.registers[x as usize].wrapping_add(nn);
            }
            (0x8, _, _, _) => {}
            (0x9, _, _, _) => {}
            (0xA, _, _, _) => {
                // ANNN: Set index register I to NNN
                println!("Handling opcode: {:#x?} - setting index register to {}", opcode, nnn);
                self.index_register = nnn;
            }
            (0xB, _, _, _) => {}
            (0xC, _, _, _) => {}
            (0xD, _, _, _) => {
                println!(
                    "Handling opcode: {:#x?}. drawing sprite of {} rows at ({}, {})",
                    opcode, n, x, y
                );
                // DXYN: draw
                // N = height of the sprite
                // X = horizontal coordinate in VX
                // Y = vertical coordinate in VY
                let x_start = self.registers[x as usize] % DISPLAY_WIDTH as u8; // X coordinate
                let y_start = self.registers[y as usize] % DISPLAY_HEIGHT as u8; // Y coordinate
                self.registers[0xF] = 0; // Set VF to 0

                for row in 0..n {
                    let y = y_start + row;
                    if y >= DISPLAY_HEIGHT as u8{
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
            (0xE, _, _, _) => {}
            (0xF, _, _, _) => {}
            _ => panic!("Unimplemented opcode: {:#x?}", opcode),
        }
    }
}
