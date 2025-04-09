// cpu.rs - defines the CPU struct that will be a part of runner
// handles all CPU-specific functionality

pub struct CPU {
    // 8-bit general-purpose registers (7 total)
    // b-l can be used together as a 16-bit register (bc, de, hl)
    pub a: u8, // Accumulator
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Special registers
    pub stack_pointer: u16, // Stack Pointer
    pub program_counter: u16, // Program Counter

    // Flags
    pub zero: bool,  // Zero flag
    pub sign: bool,  // Sign flag
    pub parity: bool,  // Parity flag
    pub carry: bool, // Carry flag
    pub aux_carry: bool, // Auxiliary carry flag

    pub interrupt_enable: bool, // interrupt enable, not part of flags register
}

impl CPU {
    /// Create a new CPU with all registers set to 0
    pub fn new() -> Self {
        CPU {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            stack_pointer: 0,
            program_counter: 0,
            zero: false,
            sign: false,
            parity: false,
            carry: false,
            aux_carry: false,
            interrupt_enable: false,
        }
    }

    // reset the CPU
    pub fn reset(&mut self) {
        self.program_counter = 0;
        self.interrupt_enable = false; // only enable interrupts after EI instr

        // reset registers
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        self.stack_pointer = 0;
        self.zero = false;
        self.sign = false;
        self.parity = false;
        self.carry = false;
        self.aux_carry = false;
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
