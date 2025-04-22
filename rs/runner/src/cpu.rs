// cpu.rs - defines the CPU struct that will be a part of runner
// handles all CPU-specific functionality
#[derive(Default, Debug, Clone, Copy)]
pub struct Flags {
    szpc: u8, // sign, zero, 0, 0, 0, parity, 0, carry
    aux_input: (u8, u8, u8, bool), // aux carry calculations input
}

impl Flags {
    // sign is bit 7 of flags
    pub fn sign(&self) -> bool { (self.szpc & 0x80) != 0 }
    // parity is bit 2 or flags
    pub fn parity(&self) -> bool { (self.szpc & 0x04) != 0 }
    // zero is bit 6 of flags
    pub fn zero(&self) -> bool { (self.szpc & 0x40) != 0 }
    // carry is bit 0 of flags
    pub fn carry(&self) -> bool { (self.szpc & 0x01) != 0 }
    // similar to carry, would be bit 5
    pub fn aux(&self) -> bool {
        let (a, b, c, f) = self.aux_input;
        f | (((a & 0xF) + (b & 0xF) + c) & 0x10 != 0)
    }

    // to support pushing of PSW to stack
    pub fn pack_flags(&self) -> u16 {
        // bit 1 is always 1
        // read aux flag and move to bit 5
        u16::from(self.szpc) | 0x0002 | ((self.aux() as u16) << 4)
    }

    // to support unpacking flags from popped PSW
    pub fn unpack_flags(&mut self, psw: u16) {
        // truncate psw into flags register, unpack
        self.szpc = (psw as u8) & 0b1100_0101;
        self.aux_input = (0, 0, 0, (psw & 0x0010) != 0);

    }
}

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
    pub stack_pointer: usize, // Stack Pointer
    pub program_counter: usize, // Program Counter

    // Flags
    pub flags: Flags,
    pub interrupt_enable: bool, // interrupt enable, not part of flags register
    pub halt: bool, // true if cpu is halted
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
            flags: Default::default(),
            interrupt_enable: false,
            halt: false,
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
        self.flags = Default::default();
        self.halt = false;
    }

    pub fn read_pc(&self) -> u16 { self.program_counter as u16 }
    pub fn read_sp(&self) -> u16 { self.stack_pointer as u16 }

    pub fn set_interrupt_enable(&mut self, enable: bool) {
        self.interrupt_enable = enable;
    }

    pub fn jump(&mut self, jump_addr: u16) {
        self.program_counter = jump_addr as usize;
    }


    // Example of how to execute a single instruction
    pub fn execute_instruction(&mut self, instruction: u8, cycle_num: u8) {
        // do nothing if halt is asserted
        if self.halt == false {
            match instruction {
                0x00 => { /* NOP (No Operation) */ }
                0x76 => { self.halt(); } // HLT (Halt)
                _ => { /* Handle other instructions */ }
            }
        }
    }

    // Halt the CPU
    pub fn halt(&mut self) {
        self.halt = true;
    }
}