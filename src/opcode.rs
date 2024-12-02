pub struct OpCode {
    pub c: u8,
    pub x: u8,
    pub y: u8,
    pub n: u8,
    pub nn: u8,
    pub nnn: u16,
}

impl OpCode {
    /// Decode the instruction to find out what the emulator should do
    pub fn decode(&opcode: &u16) -> Self {
        let c = ((opcode & 0xF000) >> 12) as u8; // first nibble (operation category)
        let x = ((opcode & 0x0F00) >> 8) as u8; // second nibble (register loop up)
        let y = ((opcode & 0x00F0) >> 4) as u8; // third nibble (register look up)
        let n = (opcode & 0x000F) as u8; // fourth nibble (a 4-bit number)
        let nn = (opcode & 0x00FF) as u8; // second byte (an 8-bit immediate number)
        let nnn = opcode & 0x0FFF; // second, third and fourth nibbles (a 12 bit immediate memory address)

        OpCode {
            c,
            x,
            y,
            n,
            nn,
            nnn,
        }
    }
}
