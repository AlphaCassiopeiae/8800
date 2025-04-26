use crate::instructions::*;
use crate::ram::RAM;
use defmt::*;

// masks for flags
const FLAG_S: u8 = 0b1000_0000; // Sign
const FLAG_Z: u8 = 0b0100_0000; // Zero
const FLAG_AC: u8 = 0b0001_0000; // Auxiliary Carry
const FLAG_P: u8 = 0b0000_0100; // Parity
const FLAG_C: u8 = 0b0000_0001; // Carry

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    M = 0b110,
    A = 0b111,
}

impl From<u8> for Reg {
    #[inline]
    fn from(x: u8) -> Reg {
        match x {
            0b000 => Reg::B,
            0b001 => Reg::C,
            0b010 => Reg::D,
            0b011 => Reg::E,
            0b100 => Reg::H,
            0b101 => Reg::L,
            0b110 => Reg::M,
            0b111 => Reg::A,
            _ => Reg::B, // doesn't really matter, will never happen
        }
    }
}

impl From<Reg> for u8 {
    #[inline]
    fn from(reg: Reg) -> u8 {
        match reg {
            Reg::B => 0b000,
            Reg::C => 0b001,
            Reg::D => 0b010,
            Reg::E => 0b011,
            Reg::H => 0b100,
            Reg::L => 0b101,
            Reg::M => 0b110,
            Reg::A => 0b111,
        }
    }
}

pub struct CPU<
C1: core::ops::AsyncFnMut(u32) -> u8,
C2: core::ops::AsyncFnMut(u32, u8),
> {
    // registers
    registers: [u8; 8],
    // special registers
    pub pc: usize,
    sp: usize,
    // halt flag
    pub halt: bool,
    // interrupt enable
    interrput_enable: bool,
    // other flags
    sign: bool,
    zero: bool,
    auxc: bool,
    parity: bool,
    carry: bool,
    // memory
    pub ram: RAM<C1, C2>,
}

impl<
C1: core::ops::AsyncFnMut(u32) -> u8,
C2: core::ops::AsyncFnMut(u32, u8),
> CPU<C1, C2>
{
    pub fn new(ram: RAM<C1, C2>) -> Self {
        Self {
            registers: [0; 8],
            pc: 0,
            sp: 0,
            halt: false,
            interrput_enable: false,
            sign: false,
            zero: false,
            auxc: false,
            parity: false,
            carry: false,
            ram,
        }
    }

    pub async fn reset(&mut self) {
        self.registers = [0; 8];
        self.pc = 0;
        self.sp = 0xFFFF; // default value for now
        self.halt = false;
        self.interrput_enable = false;
        self.sign = false;
        self.zero = false;
        self.auxc = false;
        self.parity = false;
        self.carry = false;
    }

    pub async fn read_bc(&mut self) -> u16 {
        ((self.read_reg(Reg::B).await as u16) << 8) | (self.read_reg(Reg::C).await as u16)
    }

    pub async fn read_de(&mut self) -> u16 {
        ((self.read_reg(Reg::D).await as u16) << 8) | (self.read_reg(Reg::E).await as u16)
    }

    pub fn read_hl(&mut self) -> u16 {
        // concatentation of H and L
        ((self.registers[0b100] as u16) << 8) | (self.registers[0b101] as u16)
    }

    pub async fn read_psw(&mut self) -> u16 {
        ((self.read_reg(Reg::A).await as u16) << 8) | (self.pack_flags() as u16)
    }

    pub async fn set_bc(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;

        self.set_reg(Reg::B, upper).await;
        self.set_reg(Reg::C, lower).await;
    }

    pub async fn set_de(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;

        self.set_reg(Reg::D, upper).await;
        self.set_reg(Reg::E, lower).await;
    }

    pub async fn set_hl(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;

        self.set_reg(Reg::H, upper).await;
        self.set_reg(Reg::L, lower).await;
    }

    pub async fn set_psw(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;

        self.set_reg(Reg::A, upper).await;
        // unpack u8 into flags
        self.unpack_flags(lower);
    }

    /* M Register */
    pub async fn read_m(&mut self) -> u8 {
        let addr: u16 = self.read_hl();
        // tram read request, ram for now
        self.ram.read(addr as usize).await //placeholder
    }

    pub async fn write_m(&mut self, val: u8) {
        let addr: u16 = self.read_hl();
        // tram write request
        self.ram.write(addr as usize, val).await;
    }

    pub fn halted(&self) -> bool {
        self.halt
    }
    pub fn halt(&mut self) {
        self.halt = true;
    }
    pub fn unhalt(&mut self) {
        self.halt = false;
    }

    pub async fn read_reg(&mut self, reg: Reg) -> u8 {
        match reg {
            Reg::M => self.read_m().await,
            _ => self.registers[reg as usize ^ 1],
        }
    }

    pub async fn set_reg(&mut self, reg: Reg, val: u8) {
        match reg {
            Reg::M => {
                self.write_m(val).await;
            }
            _ => {
                self.registers[reg as usize ^ 1] = val;
            }
        }
    }

    pub fn set_szp(&mut self, result: u8) {
        self.sign = (result & 0x80) != 0;
        self.zero = result == 0;
        self.parity = result.count_ones() % 2 == 0;
    }

    // packs flags into flags register format
    pub fn pack_flags(&self) -> u8 {
        let mut base: u8 = 0b0000_0010; // bit 1 is always 1

        if self.sign {
            base |= FLAG_S;
        }
        if self.zero {
            base |= FLAG_Z;
        }
        if self.auxc {
            base |= FLAG_AC;
        }
        if self.parity {
            base |= FLAG_P;
        }
        if self.carry {
            base |= FLAG_C;
        }

        base
    }

    // unpacks flags from u8
    pub fn unpack_flags(&mut self, flags: u8) {
        // u8 structure: S, Z, 0, A, 0, P, 1, C
        self.sign = (flags & 0x80) == 0x80;
        self.zero = (flags & 0x40) == 0x40;
        self.auxc = (flags & 0x10) == 0x10;
        self.parity = (flags & 0x40) == 0x40;
        self.carry = (flags & 0x01) == 0x01;
    }

    // pub fn format_flags(&self) -> String {
    //     format!(
    //         "[{}{}{}{}{}]",
    //         if self.sign { "S" } else { "." },  // Sign
    //         if self.zero { "Z" } else { "." },  // Zero
    //         if self.auxc { "A" } else { "." },  // Aux Carry
    //         if self.parity { "P" } else { "." },  // Parity
    //         if self.carry { "C" } else { "." },  // Carry
    //     )
    // }

    pub async fn fetch_instruction(&mut self) -> u8 {
        // TODO: update for making tram request
        self.ram.read(self.pc).await
    }

    // designed for use with pc (for instructions that need additional bytes)
    pub async fn fetch_word(&mut self, pc: usize) -> u16 {
        let lower: u8 = self.ram.read(pc + 1).await;
        let higher: u8 = self.ram.read(pc + 2).await;
        ((higher as u16) << 8) | (lower as u16)
    }

    pub async fn fetch_byte(&mut self, pc: usize) -> u8 {
        self.ram.read(pc + 1).await
    }

    pub async fn stack_push(&mut self, word: u16) {
        let lower: u8 = (word & 0x00FF) as u8;
        let upper: u8 = (word >> 8) as u8;

        self.sp -= 1;
        self.ram.write(self.sp, upper).await;
        self.sp -= 1;
        self.ram.write(self.sp, lower).await;
    }

    pub async fn stack_pop(&mut self) -> u16 {
        let lower: u8 = self.ram.read(self.sp).await;
        self.sp += 1;
        let upper: u8 = self.ram.read(self.sp).await;
        self.sp += 1;

        ((upper as u16) << 8) | (lower as u16)
    }

    pub async fn execute_instruction(&mut self, instruction: u8) {
        info!("instruction: 0x{:X}\r", instruction);
        let info: InstructionInfo = INSTRUCTION_TABLE[usize::from(instruction)];
        info!("info: {}\r", info);

        // next_pc logic
        // for instructions that need bytes of data, next_pc will be updated differently
        let mut next_pc: usize = self.pc + 1;

        match instruction {
            /* NOP Opcodes */
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 => { /* NOP */ }
            /* Increments/Decrements */
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                // INR, look at bits 5-3 for register number (5 cycles)
                let reg_mask: u8 = 0b0011_1000;
                let reg: Reg = Reg::from((instruction & reg_mask) >> 3);

                let val: u8 = self.read_reg(reg).await;
                let result: u8 = val.wrapping_add(1);
                self.set_reg(reg, result).await;

                // use result to update flags (S, Z, A, P)
                self.set_szp(result);
                // aux_carry
                self.auxc = (val & 0x0F) == 0x0F;
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                // DCR, look at bits 5-3 for register number (5 cycles)
                let reg_mask: u8 = 0b0011_1000;
                let reg: Reg = Reg::from((instruction & reg_mask) >> 3);

                let val: u8 = self.read_reg(reg).await;
                let result: u8 = val.wrapping_sub(1);
                self.set_reg(reg, result).await;

                // use result to update flags (S, Z, A, P)
                self.set_szp(result);
                // aux_carry
                self.auxc = (val & 0x0F) == 0x00;
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                // INX
                let pair: u8 = (instruction & 0x30) >> 4;

                match pair {
                    0b00 => {
                        // pair BC
                        let val: u16 = self.read_bc().await;
                        self.set_bc(val.wrapping_add(1)).await;
                    }
                    0b01 => {
                        // pair DE
                        let val: u16 = self.read_de().await;
                        self.set_de(val.wrapping_add(1)).await;
                    }
                    0b10 => {
                        // pair HL
                        let val: u16 = self.read_hl();
                        self.set_hl(val.wrapping_add(1)).await;
                    }
                    0b11 => {
                        // SP
                        self.sp = self.sp.wrapping_add(1);
                    }
                    _ => { /* invalid pair, shouldn't be possible */ }
                }
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                // DCX
                let pair: u8 = (instruction & 0x30) >> 4;

                match pair {
                    0b00 => {
                        // pair BC
                        let val: u16 = self.read_bc().await;
                        self.set_bc(val.wrapping_sub(1)).await;
                    }
                    0b01 => {
                        // pair DE
                        let val: u16 = self.read_de().await;
                        self.set_de(val.wrapping_sub(1)).await;
                    }
                    0b10 => {
                        // pair HL
                        let val: u16 = self.read_hl();
                        self.set_hl(val.wrapping_sub(1)).await;
                    }
                    0b11 => {
                        // SP
                        self.sp = self.sp.wrapping_sub(1);
                    }
                    _ => { /* invalid pair, shouldn't be possible */ }
                }
            }
            /* Misc Instructions */
            0x07 => {
                // RLC
                let a: u8 = self.read_reg(Reg::A).await;
                let msb: u8 = (a & 0x80) >> 7;
                self.set_reg(Reg::A, (a << 1) | msb).await;
                self.carry = msb == 1;
            }
            0x17 => {
                // RAL
                let mut carry: u8 = 0;
                if self.carry {
                    carry = 1;
                }
                let a: u8 = self.read_reg(Reg::A).await;
                let msb: u8 = (a & 0x80) >> 7;
                self.set_reg(Reg::A, (a << 1) | carry).await;
                self.carry = msb == 1;
            }
            0x27 => {
                // DAA
                let a: u8 = self.read_reg(Reg::A).await;

                let mut adjustment: u8 = 0;

                if (a & 0x0F) > 9 || self.auxc {
                    adjustment += 0x06;
                    self.auxc = true;
                }

                if (a > 0x99) || self.carry {
                    adjustment += 0x60;
                    self.carry = true;
                }

                let result: u8 = a.wrapping_add(adjustment);
                self.set_reg(Reg::A, result).await;
                self.set_szp(result);
            }
            0x37 => {
                // STC
                self.carry = true;
            }
            0x0F => {
                // RRC
                let a: u8 = self.read_reg(Reg::A).await;
                let lsb: u8 = a & 0x01;
                self.set_reg(Reg::A, (a >> 1) | (lsb << 7)).await;
                self.carry = lsb == 1;
            }
            0x1F => {
                // RAR
                let mut carry: u8 = 0;
                if self.carry {
                    carry = 1;
                }
                let a: u8 = self.read_reg(Reg::A).await;
                let lsb: u8 = a & 0x01;
                self.set_reg(Reg::A, (a >> 1) | (carry << 7)).await;
                self.carry = lsb == 1;
            }
            0x2F => {
                // CMA
                let value = self.read_reg(Reg::A).await;
                self.set_reg(Reg::A, !value).await;
            }
            0x3F => {
                // CMC
                self.carry = !self.carry;
            }
            /* Loads and Stores */
            0x02 | 0x12 => {
                // STAX, no flags affected
                // store val from accumulator at addr housed in reg pair
                let pair: u8 = (instruction & 0x10) >> 4;
                let val: u8 = self.read_reg(Reg::A).await;

                match pair {
                    0 => {
                        // pair bc
                        let addr: u16 = self.read_bc().await;
                        self.ram.write(addr as usize, val).await;
                    }
                    1 => {
                        // pair de
                        let addr: u16 = self.read_de().await;
                        self.ram.write(addr as usize, val).await;
                    }
                    _ => { /* Invalid Register Pair */ }
                }
            }
            0x0A | 0x1A => {
                // LDAX, no flags affected
                // load val from addr housed in reg pair, put in accumulator
                let pair: u8 = (instruction & 0x10) >> 4;

                match pair {
                    0 => {
                        // pair bc
                        let addr: u16 = self.read_bc().await;
                        let value = self.ram.read(addr as usize).await;
                        self.set_reg(Reg::A, value).await;
                    }
                    1 => {
                        // pair de
                        let addr: u16 = self.read_de().await;
                        let value = self.ram.read(addr as usize).await;
                        self.set_reg(Reg::A, value).await;
                    }
                    _ => { /* Invalid Register Pair */ }
                }
            }
            0x22 => {
                // SHLD a16
                // H -> MEM[a16], L -> MEM[a16+1]
                let addr: u16 = self.fetch_word(self.pc).await;

                let mut value = self.read_reg(Reg::L).await;
                self.ram.write(addr as usize, value).await;

                value = self.read_reg(Reg::L).await;
                self.ram.write((addr + 1) as usize, value).await;

                next_pc = self.pc + 3;
            }
            0x2A => {
                // LHLD a16
                // MEM[a16] -> L, MEM[a16+1] -> H
                let addr: u16 = self.fetch_word(self.pc).await;

                let l_byte: u8 = self.ram.read(addr as usize).await;
                let h_byte: u8 = self.ram.read((addr + 1) as usize).await;

                self.set_reg(Reg::L, l_byte).await;
                self.set_reg(Reg::H, h_byte).await;

                next_pc = self.pc + 3;
            }
            0x32 => {
                // STA a16
                let addr: u16 = self.fetch_word(self.pc).await;

                let value = self.read_reg(Reg::A).await;
                self.ram.write(addr as usize, value).await;

                next_pc = self.pc + 3;
            }
            0x3A => {
                // LDA a16
                let addr: u16 = self.fetch_word(self.pc).await;

                let value = self.ram.read(addr as usize).await;
                self.set_reg(Reg::A, value).await;

                next_pc = self.pc + 3;
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                // LXI rp, d16
                let lower: u8 = self.fetch_byte(self.pc).await;
                let upper: u8 = self.fetch_byte(self.pc + 1).await;

                // rp encoded in bits 5-4 of opcode
                let code: u8 = (instruction & 0x30) >> 4;
                match code {
                    0x00 => {
                        // BC
                        self.set_reg(Reg::B, upper).await;
                        self.set_reg(Reg::C, lower).await;
                    }
                    0x01 => {
                        // DE
                        self.set_reg(Reg::D, upper).await;
                        self.set_reg(Reg::E, lower).await;
                    }
                    0x02 => {
                        // HL
                        self.set_reg(Reg::H, upper).await;
                        self.set_reg(Reg::L, lower).await;
                    }
                    0x03 => {
                        // SP
                        let concat: u16 = ((upper as u16) << 8) | (lower as u16);
                        self.sp = concat as usize;
                    }
                    _ => {}
                }

                next_pc = self.pc + 3;
            }
            /* MOV Instructions */
            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                // MVI reg, d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                // reg encoded in bits 5-3 of opcode
                let reg_code: u8 = (instruction & 0x38) >> 3;
                let reg: Reg = Reg::from(reg_code);
                self.set_reg(reg, byte).await;

                next_pc = self.pc + 2;
            }
            0x40..=0x75 | 0x77..=0x7F => {
                // MOV instructions
                let src: Reg = Reg::from(instruction & 0b111);
                let dst: Reg = Reg::from((instruction >> 3) & 0b111);

                if src == Reg::M {
                    // read mem at addr HL, store value in dst
                    let value = self.read_m().await;
                    self.set_reg(dst, value).await;
                } else if dst == Reg::M {
                    // store value in src at mem addr HL
                    let value = self.read_reg(src).await;
                    self.write_m(value).await;
                } else {
                    // reg -> reg mov
                    let value: u8 = self.read_reg(src).await;
                    self.set_reg(dst, value).await;
                }
            }
            /* HLT */
            0x76 => {
                self.halt = true;
            }
            /* 8-bit Register Arithmetic */
            0x80..=0x87 => {
                // ADD instructions
                let src_reg: Reg = Reg::from(instruction & 0b111);

                let a: u8 = self.read_reg(Reg::A).await;
                let b: u8 = self.read_reg(src_reg).await;
                let result: u8 = a.wrapping_add(b);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = (a as u16 + b as u16) > 0x00FF;
                self.auxc = ((a & 0x0F) + (b & 0x0F)) & 0x10 == 0x10;
            }
            0x88..=0x8F => {
                // ADC instructions
                let src_reg: Reg = Reg::from(instruction & 0b111);

                let mut carry: u8 = 0;
                if self.carry {
                    carry = 1;
                }

                let a: u8 = self.read_reg(Reg::A).await;
                let b: u8 = self.read_reg(src_reg).await;
                let result: u8 = a.wrapping_add(b).wrapping_add(carry);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = (a as u16 + b as u16 + carry as u16) > 0x00FF;
                self.auxc = ((a & 0x0F) + (b & 0x0F) + carry) > 0xF;
            }
            0x90..=0x97 => {
                // SUB instructions
                let src_reg: Reg = Reg::from(instruction & 0b0000_0111);

                let a: u8 = self.read_reg(Reg::A).await;
                let b: u8 = self.read_reg(src_reg).await;
                let result: u8 = a.wrapping_sub(b);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = a < b;
                self.auxc = (a & 0x0F) < (b & 0x0F);
            }
            0x98..=0x9F => {
                // SBB instructions
                let src_reg: Reg = Reg::from(instruction & 0b0000_0111);

                let mut carry: u8 = 0;
                if self.carry {
                    carry = 1;
                }

                let a: u8 = self.read_reg(Reg::A).await;
                let b: u8 = self.read_reg(src_reg).await;
                let result: u8 = a.wrapping_sub(b).wrapping_sub(carry);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = a < (b + carry);
                self.auxc = (a & 0x0F) < ((b & 0x0F) + carry);
            }
            0xA0..=0xA7 => {
                // ANA instructions
                let src_reg: Reg = Reg::from(instruction & 0b0000_0111);

                let result: u8 = self.read_reg(Reg::A).await & self.read_reg(src_reg).await;
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = false;
                self.auxc = true;
            }
            0xA8..=0xAF => {
                // XRA instructions
                let src_reg: Reg = Reg::from(instruction & 0b0000_0111);

                let result: u8 = self.read_reg(Reg::A).await ^ self.read_reg(src_reg).await;
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = false;
                self.auxc = false;
            }
            0xB0..=0xB7 => {
                // ORA instructions
                let src_reg: Reg = Reg::from(instruction & 0b0000_0111);

                let result: u8 = self.read_reg(Reg::A).await | self.read_reg(src_reg).await;
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = false;
                self.auxc = false;
            }
            0xB8..=0xBF => {
                // CMP instructions
                // just updates flags after subraction, A remains untouched
                let src_reg: Reg = Reg::from(instruction & 0b0000_0111);

                let a: u8 = self.read_reg(Reg::A).await;
                let b: u8 = self.read_reg(src_reg).await;
                let result: u8 = a.wrapping_sub(b);

                self.set_szp(result);
                self.carry = a < b;
                self.auxc = (a & 0x0F) < (b & 0x0F);
            }
            0xC6 => {
                // ADI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let result: u8 = a.wrapping_add(byte);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = (a as u16 + byte as u16) > 0x00FF;
                self.auxc = ((a & 0x0F) + (byte & 0x0F)) & 0x10 == 0x10;

                next_pc = self.pc + 2;
            }
            0xD6 => {
                // SUI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let result: u8 = a.wrapping_sub(byte);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = a < byte;
                self.auxc = (a & 0x0F) < (byte & 0x0F);

                next_pc = self.pc + 2;
            }
            0xE6 => {
                // ANI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let result: u8 = a & byte;
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = false;
                self.auxc = true;

                next_pc = self.pc + 2;
            }
            0xF6 => {
                // ORI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let result: u8 = a & byte;
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = false;
                self.auxc = false;

                next_pc = self.pc + 2;
            }
            0xCE => {
                // ACI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let mut carry: u8 = 0;
                if self.carry {
                    carry = 1;
                }

                let result: u8 = a.wrapping_add(byte).wrapping_add(carry);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = (a as u16 + byte as u16 + carry as u16) > 0x00FF;
                self.auxc = ((a & 0x0F) + (byte & 0x0F) + carry) > 0xF;

                next_pc = self.pc + 2;
            }
            0xDE => {
                // SBI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let mut carry: u8 = 0;
                if self.carry {
                    carry = 1;
                }

                let result: u8 = a.wrapping_sub(byte).wrapping_sub(carry);
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = a < (byte + carry);
                self.auxc = (a & 0x0F) < ((byte & 0x0F) + carry);

                next_pc = self.pc + 2;
            }
            0xEE => {
                // XRI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let result: u8 = a ^ byte;
                self.set_reg(Reg::A, result).await;

                self.set_szp(result);
                self.carry = false;
                self.auxc = false;

                next_pc = self.pc + 2;
            }
            0xFE => {
                // CPI d8
                let byte: u8 = self.fetch_byte(self.pc).await;
                let a: u8 = self.read_reg(Reg::A).await;

                let result: u8 = a.wrapping_sub(byte);

                self.set_szp(result);
                self.carry = a < byte;
                self.auxc = (a & 0x0F) < (byte & 0x0F);

                next_pc = self.pc + 2;
            }
            /* Jumps and Conditional Jumps */
            0xC2 => {
                // JNZ a16, conditional jump if zero flag is 0
                if !self.zero {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xC3 | 0xCB => {
                // JMP a16, needs 2 bytes (lower, higher) from pc+1 and pc+2
                let addr: u16 = self.fetch_word(self.pc).await;
                next_pc = addr as usize;
            }
            0xCA => {
                // JZ a16
                if self.zero {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xD2 => {
                // JNC a16, conditional jump if carry = false
                if !self.carry {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xDA => {
                // JC a16
                if self.carry {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xE2 => {
                // JPO a16, conditional jump if parity odd (false)
                if !self.parity {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xEA => {
                // JPE a16
                if self.parity {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xF2 => {
                // JP a16, conditional jump if sign positive
                if !self.sign {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xFA => {
                // JM a16
                if self.sign {
                    let addr: u16 = self.fetch_word(self.pc).await;
                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            /* PUSH and POP */
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                // POP instructions
                let word: u16 = self.stack_pop().await;

                let pair: u8 = (instruction & 0b00110000) >> 4;

                match pair {
                    0b00 => {
                        // pair BC
                        self.set_bc(word).await;
                    }
                    0b01 => {
                        // pair DE
                        self.set_de(word).await;
                    }
                    0b10 => {
                        // pair HL
                        self.set_hl(word).await;
                    }
                    0b11 => {
                        // PSW (A, flags)
                        self.set_psw(word).await;
                    }
                    _ => { /* invalid pair, shouldn't be possible */ }
                }
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                // PUSH instructions
                let pair: u8 = (instruction & 0b00110000) >> 4;
                let mut word: u16 = 0;

                match pair {
                    0b00 => {
                        // pair BC
                        word = self.read_bc().await;
                    }
                    0b01 => {
                        // pair DE
                        word = self.read_de().await;
                    }
                    0b10 => {
                        // pair HL
                        word = self.read_hl();
                    }
                    0b11 => {
                        // PSW (A, flags)
                        word = self.read_psw().await;
                    }
                    _ => { /* invalid pair, shouldn't be possible */ }
                }

                self.stack_push(word).await;
            }
            /* Interrupt Enable/Disable */
            0xF3 => {
                // DI (4 cycles)
                self.interrput_enable = false;
            }
            0xFB => {
                // EI (4 cycles)
                self.interrput_enable = true;
            }
            /* CALL instructions */
            0xCD | 0xDD | 0xED | 0xFD => {
                // CALL a16
                // push pc+3 to stack, set pc to a16
                let addr: u16 = self.fetch_word(self.pc).await;

                let store_pc: u16 = (self.pc + 3) as u16;

                self.stack_push(store_pc).await;

                next_pc = addr as usize;
            }
            0xC4 => {
                // CNZ a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if !self.zero {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xD4 => {
                // CNC a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if !self.carry {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xE4 => {
                // CPO a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if !self.parity {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xF4 => {
                // CP a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if !self.sign {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xCC => {
                // CZ a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if self.zero {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xDC => {
                // CC a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if self.carry {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xEC => {
                // CPE a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if self.parity {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            0xFC => {
                // CM a16
                let addr: u16 = self.fetch_word(self.pc).await;

                if self.sign {
                    let store_pc: u16 = (self.pc + 3) as u16;
                    self.stack_push(store_pc).await;

                    next_pc = addr as usize;
                } else {
                    next_pc = self.pc + 3;
                }
            }
            /* RET instructions */
            0xC9 | 0xD9 => {
                // RET, pop 2 bytes off of stack and set as new pc
                let new_pc: u16 = self.stack_pop().await;
                next_pc = new_pc as usize;
            }
            0xC0 => {
                // RNZ
                if !self.zero {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            0xD0 => {
                // RNC
                if !self.carry {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            0xE0 => {
                // RPO
                if !self.parity {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            0xF0 => {
                // RP
                if !self.sign {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            0xC8 => {
                // RZ
                if self.zero {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            0xD8 => {
                // RC
                if self.carry {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            0xE8 => {
                // RPE
                if self.parity {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            0xF8 => {
                // RM
                if self.sign {
                    let new_pc: u16 = self.stack_pop().await;
                    next_pc = new_pc as usize;
                }
            }
            /* RST instructions */
            0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                // RST
                let new_pc: u16 = (instruction & 0x38) as u16;

                // push old pc to stack (lower then upper)
                self.stack_push(self.pc as u16).await;
                next_pc = new_pc as usize;
            }
            /* Misc Instructions */
            0xE3 => {
                // XTHL
                let temp_l: u8 = self.ram.read(self.sp).await;
                let temp_h: u8 = self.ram.read(self.sp + 1).await;

                let mut value = self.read_reg(Reg::L).await;
                self.ram.write(self.sp, value).await;

                value = self.read_reg(Reg::H).await;
                self.ram.write(self.sp + 1, value).await;

                self.set_reg(Reg::H, temp_h).await;
                self.set_reg(Reg::L, temp_l).await;
            }
            0xE9 => {
                // PCHL (HL -> PC)
                next_pc = self.read_hl() as usize;
            }
            0xEB => {
                // XCHG
                // swaps contents of DE and HL (D <-> H, E <-> L)
                let mut temp = self.read_reg(Reg::D).await;
                let mut value = self.read_reg(Reg::H).await;
                self.set_reg(Reg::D, value).await;
                self.set_reg(Reg::H, temp).await;

                temp = self.read_reg(Reg::E).await;
                value = self.read_reg(Reg::L).await;
                self.set_reg(Reg::E, value).await;
                self.set_reg(Reg::L, temp).await;
            }
            0xF9 => {
                // SPHL (HL -> SP)
                self.sp = self.read_hl() as usize;
            }
            _ => {}
        }
        self.pc = next_pc;
    }

    pub async fn show_state(&mut self) {
        info!("------------------------------\r");
        info!("Registers:\r");
        info!("  A: {}\r", self.read_reg(Reg::A).await);
        info!(
            "  B: {}  C: {}\r",
            self.read_reg(Reg::B).await,
            self.read_reg(Reg::C).await
        );
        info!(
            "  D: {}  E: {}\r",
            self.read_reg(Reg::D).await,
            self.read_reg(Reg::E).await
        );
        info!(
            "  H: {}  L: {}\r",
            self.read_reg(Reg::H).await,
            self.read_reg(Reg::L).await
        );
        info!("  FLAGS: {}\r", self.pack_flags());
        info!("------------------------------\r");
    }

    // pub async fn disable_ram(&mut self) {
    //     self.ram.disable();
    // }
}

// impl fmt::Display for CPU {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(f, "----------------------------------\r")?;
//         writeln!(f, "CPU State:\r")?;
//         writeln!(f, "  PC: 0x{:04X}\r", self.pc)?;
//         writeln!(f, "  SP: 0x{:04X}\r", self.sp)?;
//         writeln!(f, "  Registers:\r")?;
//         writeln!(f, "    A: 0x{:02X}\r", self.registers[0b111 as usize ^ 1])?;
//         writeln!(f, "    B: 0x{:02X}  C: 0x{:02X}\r", self.registers[0b000 as usize ^ 1], self.registers[0b001 as usize ^ 1])?;
//         writeln!(f, "    D: 0x{:02X}  E: 0x{:02X}\r", self.registers[0b010 as usize ^ 1], self.registers[0b011 as usize ^ 1])?;
//         writeln!(f, "    H: 0x{:02X}  L: 0x{:02X}\r", self.registers[0b100 as usize ^ 1], self.registers[0b101 as usize ^ 1])?;
//         writeln!(f, "    FLAGS: {}\r", self.format_flags())?;
//         writeln!(f, "----------------------------------\r")
//     }
// }
