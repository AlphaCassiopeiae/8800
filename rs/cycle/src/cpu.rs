struct CPU {
    // 8-bit general-purpose registers
    a: u8, // Accumulator
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    // Special registers
    sp: u16, // Stack Pointer
    pc: u16, // Program Counter

    // Flags
    z: bool,  // Zero flag
    s: bool,  // Sign flag
    p: bool,  // Parity flag
    cy: bool, // Carry flag
    ac: bool, // Auxiliary carry flag
}

impl CPU {
    /// Create a new CPU with all registers set to 0
    fn new() -> Self {
        CPU {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            z: false,
            s: false,
            p: false,
            cy: false,
            ac: false,
        }
    }

    /// Example of how to execute a single instruction
    fn execute_instruction(&mut self, instruction: u8) {
        match instruction {
            0x00 => { /* NOP (No Operation) */ }
            0x04 => {
                // MVI

            }
            0x76 => { self.halt(); } // HLT (Halt)
            _ => { /* Handle other instructions */ }
        }
    }

    /// Halt the CPU
    fn halt(&mut self) {
        // Logic for halting the CPU
        // For example, setting a halt flag or stopping the main loop
    }
}




fn get mem() // write to memory on the s100 bus


fn write mem() // write to memory on the s100 bus
