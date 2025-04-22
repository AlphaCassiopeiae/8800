use std::fmt;
use crate::instructions::*;
use crate::ram::*;

// masks for flags
const FLAG_S: u8  = 0b1000_0000; // Sign
const FLAG_Z: u8  = 0b0100_0000; // Zero
const FLAG_AC: u8 = 0b0001_0000; // Auxiliary Carry
const FLAG_P: u8  = 0b0000_0100; // Parity
const FLAG_C: u8 = 0b0000_0001; // Carry

#[derive(Copy, Clone, Debug)]
pub enum Reg {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    // M = 0b110 but is handled separately
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
            0b110 => panic!("M encoding for Reg made it past decoder"),
            0b111 => Reg::A,
            _ => panic!("Encoding for Reg out of range: {}", x),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CPU {
    // registers
    registers: [u8; 8],
    // special registers
    pc: usize,
    sp: usize,
    // halt flag
    halt: bool,
    // interrupt enable
    interrput_enable: bool,
    // other flags
    sign: bool,
    zero: bool,
    auxc: bool,
    parity: bool,
    carry: bool,
    // memory
    ram: RAM,
}

impl CPU {
    pub fn new(mut _ram: RAM) -> Self {
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
            ram: _ram,
        }
    }

    pub fn reset(&mut self) {
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

    pub fn read_bc(&self) -> u16 {
        ((self.registers[0b000] as u16) << 8) | (self.registers[0b001] as u16)
    }
    
    pub fn read_de(&self) -> u16 {
        ((self.registers[0b010] as u16) << 8) | (self.registers[0b011] as u16)
    }
    
    pub fn read_hl(&self) -> u16 {
        // concatentation of H and L
        ((self.registers[0b100] as u16) << 8) | (self.registers[0b101] as u16)
    }

    pub fn read_psw(&self) -> u16 {
        ((self.read_reg(0b111) as u16) << 8) | (self.pack_flags() as u16)
    }

    pub fn set_bc(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;

        self.registers[0b000] = upper;
        self.registers[0b001] = lower;
    }

    pub fn set_de(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;
        
        self.registers[0b010] = upper;
        self.registers[0b011] = lower;
    }
    
    pub fn set_hl(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;

        self.registers[0b100] = upper;
        self.registers[0b101] = lower;
    }

    pub fn set_psw(&mut self, val: u16) {
        let upper: u8 = ((val & 0xFF00) >> 8) as u8;
        let lower: u8 = (val & 0x00FF) as u8;

        self.registers[0b111] = upper;
        // unpack u8 into flags
        self.unpack_flags(lower);

    }

    /* M Register */
    pub fn read_m(&self) -> u8 {
        let addr: u16 = self.read_hl();
        // tram read request, ram for now
        self.ram.read(addr as usize) //placeholder
    }

    pub fn write_m(&mut self, val: u8) {
        let addr: u16 = self.read_hl();
        // tram write request
        self.ram.write(addr as usize, val);
    }

    pub fn read_pc(&self) -> u16 {self.pc as u16}
    pub fn halted(&self) -> bool {self.halt}
    // These flag reading functions can probably be removed
    pub fn read_sign(&self) -> bool {self.sign}
    pub fn read_zero(&self) -> bool {self.zero}
    pub fn read_auxc(&self) -> bool {self.auxc}
    pub fn read_parity(&self) -> bool {self.parity}
    pub fn read_carry(&self) -> bool {self.carry}
    pub fn set_pc(&mut self, val: u16) {self.pc = val as usize;}
    pub fn set_sp(&mut self, val: u16) {self.sp = val as usize;}

    pub fn read_reg(&self, reg: u8) -> u8 {
        match reg {
            0b000..=0b101 | 0b111 => {
                self.registers[reg as usize ^ 1]
            }
            0b110 => {
                self.read_m()
            }
            _ => {0} // invalid register
        }
    }

    pub fn set_reg(&mut self, reg: u8, val: u8) {
        match reg {
            0b000..=0b101 | 0b111 => {
                self.registers[reg as usize ^ 1] = val;
            }
            0b110 => {
                self.write_m(val);
            }
            _ => {/* invalid register */}
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

        if self.sign   {base |= FLAG_S;}
        if self.zero   {base |= FLAG_Z;}
        if self.auxc   {base |= FLAG_AC;}
        if self.parity {base |= FLAG_P;}
        if self.carry  {base |= FLAG_C;}

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

    pub fn format_flags(&self) -> String {
        format!(
            "[{}{}{}{}{}]",
            if self.sign { "S" } else { "." },  // Sign
            if self.zero { "Z" } else { "." },  // Zero
            if self.auxc { "A" } else { "." },  // Aux Carry
            if self.parity { "P" } else { "." },  // Parity
            if self.carry { "C" } else { "." },  // Carry
        )
    }

    pub fn fetch_instruction(&self) -> u8 {
        // TODO: update for making tram request
        self.ram.read(self.pc)
    }

    pub fn execute_instruction(&mut self, instruction: u8) {
        println!("instruction: 0x{:X}\r", instruction);
        let info: InstructionInfo = INSTRUCTION_TABLE[usize::from(instruction)];
        println!("info: {}\r", info);

        // next_pc logic
        // for instructions that need bytes of data, next_pc will be updated differently
        let mut next_pc: usize = self.pc + 1;

        match instruction {
            // do simple instructions first
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 => {/* NOP */}
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                // INR, look at bits 5-3 for register number (5 cycles)
                // increment register, update CPU flags except carry
                // make sure this takes 5 cycles (M takes 10 cycles)
                let reg_mask: u8 = 0b0011_1000;
                let reg: u8 = (instruction & reg_mask) >> 3;

                let val: u8 = self.read_reg(reg);
                let result: u8 = val.wrapping_add(1);
                self.set_reg(reg, result);

                // use result to update flags (S, Z, A, P)
                self.set_szp(result);
                // aux_carry
                self.auxc = (val & 0x0F) == 0x0F;

            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                // DCR, look at bits 5-3 for register number (5 cycles)
                // decrement register, update CPU flags except carry
                // make sure this takes 5 cycles (M takes 10 cycles)
                let reg_mask: u8 = 0b0011_1000;
                let reg: u8 = (instruction & reg_mask) >> 3;

                let val: u8 = self.read_reg(reg);
                let result: u8 = val.wrapping_sub(1);
                self.set_reg(reg, result);

                // use result to update flags (S, Z, A, P)
                self.set_szp(result);
                // aux_carry
                self.auxc = (val & 0x0F) == 0x00;
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                // INX
                let pair: u8 = (instruction & 0b00110000) >> 4;

                match pair {
                    0b00 => {
                        // pair BC
                        let val: u16 = self.read_bc();
                        self.set_bc(val.wrapping_add(1));
                    }
                    0b01 => {
                        // pair DE
                        let val: u16 = self.read_de();
                        self.set_de(val.wrapping_add(1));
                    }
                    0b10 => {
                        // pair HL
                        let val: u16 = self.read_hl();
                        self.set_hl(val.wrapping_add(1));
                    }
                    0b11 => {
                        // SP
                        self.sp = self.sp.wrapping_add(1);
                    }
                    _ => {/* invalid pair, shouldn't be possible */}
                }
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                // DCX
                let pair: u8 = (instruction & 0b00110000) >> 4;

                match pair {
                    0b00 => {
                        // pair BC
                        let val: u16 = self.read_bc();
                        self.set_bc(val.wrapping_sub(1));
                    }
                    0b01 => {
                        // pair DE
                        let val: u16 = self.read_de();
                        self.set_de(val.wrapping_sub(1));
                    }
                    0b10 => {
                        // pair HL
                        let val: u16 = self.read_hl();
                        self.set_hl(val.wrapping_sub(1));
                    }
                    0b11 => {
                        // SP
                        self.sp = self.sp.wrapping_sub(1);
                    }
                    _ => {/* invalid pair, shouldn't be possible */}
                }
            }
            0x07 => {
                // RLC
                let a: u8 = self.read_reg(0b111);
                let msb: u8 = (a & 0x80) >> 7;
                self.set_reg(0b111, (a << 1) | msb);
                self.carry = msb == 1;
            }
            0x17 => {
                // RAL
                let mut carry: u8 = 0;
                if self.carry {carry = 1;}
                let a: u8 = self.read_reg(0b111);
                let msb: u8 = (a & 0x80) >> 7;
                self.set_reg(0b111, (a << 1) | carry);
                self.carry = msb == 1;
            }
            0x27 => {
                // DAA
            }
            0x37 => {
                // STC
            }
            0x0F => {
                // RRC
                let a: u8 = self.read_reg(0b111);
                let lsb: u8 = a & 0x01;
                self.set_reg(0b111, (a >> 1) | (lsb << 7));
                self.carry = lsb == 1;
            }
            0x1F => {
                // RAR
                let mut carry: u8 = 0;
                if self.carry {carry = 1;}
                let a: u8 = self.read_reg(0b111);
                let lsb: u8 = a & 0x01;
                self.set_reg(0b111, (a >> 1) | (carry << 7));
                self.carry = lsb == 1;
            }
            0x2F => {
                // CMA
                self.set_reg(0b111, !self.read_reg(0b111));
            }
            0x3F => {
                // CMC
                self.carry = !self.carry;
            }
            0x02 | 0x12 => {
                // STAX, no flags affected
                // store val from accumulator at addr housed in reg pair
                let pair: u8 = (instruction & 0x10) >> 4;
                let val: u8 = self.read_reg(0b111);

                match pair {
                    0 => {
                        // pair bc
                        let addr: u16 = self.read_bc();
                        self.ram.write(addr as usize, val);
                    }
                    1 => {
                        // pair de
                        let addr: u16 = self.read_de();
                        self.ram.write(addr as usize, val);
                    }
                    _ => {/* Invalid Register Pair */}
                }
            }
            0x0A | 0x1A => {
               // LDAX, no flags affected
               // load val from addr housed in reg pair, put in accumulator
                let pair: u8 = (instruction & 0x10) >> 4;

                match pair {
                    0 => {
                        // pair bc
                        let addr: u16 = self.read_bc();
                        self.set_reg(0b111, self.ram.read(addr as usize));
                    }
                    1 => {
                        // pair de
                        let addr: u16 = self.read_de();
                        self.set_reg(0b111, self.ram.read(addr as usize));
                    }
                    _ => {/* Invalid Register Pair */}
                }
            }
            0x32 => {
                // STA a16
                let lower: u8 = self.ram.read(self.pc + 1);
                let upper: u8 = self.ram.read(self.pc + 2);
                let addr: u16 = ((upper as u16) << 8) | (lower as u16);
                self.ram.write(addr as usize, self.read_reg(0b111));
                // no flags affected, next pc is pc + 3
                next_pc = self.pc + 3;
            }
            0x3A => {
                // LDA a16
                // bytes housed at pc + 1 (low) and pc + 2 (high)
                // [high, low] is address to load from
                let lower: u8 = self.ram.read(self.pc + 1);
                let upper: u8 = self.ram.read(self.pc + 2);
                let addr: u16 = ((upper as u16) << 8) | (lower as u16);
                self.set_reg(0b111, self.ram.read(addr as usize));
                // no flags affected
                next_pc = self.pc + 3;
            }
            0x40..=0x75 | 0x77..=0x7F => { // MOV instructions
                let src: u8 = instruction & 0b111;
                let dst: u8 = (instruction >> 3) & 0b111;
            
                if src == 0b101 {
                    // read mem at addr HL, store value in dst
                    // takes 7 cycles
                    self.set_reg(dst, self.read_m());
                }
                else if dst == 0b101 {
                    // store value in src at mem addr HL
                    // takes 7 cycles
                    self.write_m(self.read_reg(src));
                }
                else {
                    // reg -> reg mov
                    // takes 5 cycles
                    let value: u8 = self.read_reg(src);
                    self.set_reg(dst, value);
                }
            }
            0x76 => {self.halt = true;}
            0x80..=0x87 => {
                // ADD instructions
                let src_reg: u8 = instruction & 0b111;

                let a: u8 = self.read_reg(0b111);
                let b: u8 = self.read_reg(src_reg);
                let result: u8 = a.wrapping_add(b);
                self.set_reg(0b111, result);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved

                // update flags
                self.set_szp(result);
                // carry and aux_carry
                self.carry = (a as u16 + b as u16) > 0x00FF;
                self.auxc = ((a & 0x0F) + (b & 0x0F)) & 0x10 == 0x10;
            }
            0x88..=0x8F => {
                // ADC instructions
                let src_reg: u8 = instruction & 0b111;

                let mut carry: u8 = 0;
                if self.carry {carry = 1;}

                let a: u8 = self.read_reg(0b111);
                let b: u8 = self.read_reg(src_reg);
                let result: u8 = a.wrapping_add(b).wrapping_add(carry);
                self.set_reg(0b111, result);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved

                // update flags
                self.set_szp(result);
                self.carry = (a as u16 + b as u16) > 0x00FF;
                self.auxc = ((a & 0x0F) + (b & 0x0F)) & 0x10 == 0x10;
            }
            0x90..=0x97 => {
                // SUB instructions
                let src_reg: u8 = instruction & 0b0000_0111;

                let a: u8 = self.read_reg(0b111);
                let b: u8 = self.read_reg(src_reg);
                let result: u8 = a.wrapping_sub(b);
                self.set_reg(0b111, result);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved

                // update flags
                self.set_szp(result);
                // carry indicates borrow occurred
                self.carry = a < b;
                // aux carry
                self.auxc = (a & 0x0F) < (b & 0x0F)
            }
            0x98..=0x9F => {
                // SBB instructions
                let src_reg: u8 = instruction & 0b0000_0111;

                let mut carry: u8 = 0;
                if self.carry {carry = 1;}

                let a: u8 = self.read_reg(0b111);
                let b: u8 = self.read_reg(src_reg);
                let result: u8 = a.wrapping_sub(b).wrapping_sub(carry);
                self.set_reg(0b111, result);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved
                
                // update flags
                self.set_szp(result);
                self.carry = a < (b + carry);
                self.auxc = (a & 0x0F) < ((b & 0x0F) + carry)
            }
            0xA0..=0xA7 => {
                // ANA instructions
                let src_reg: u8 = instruction & 0b0000_0111;

                let result: u8 = self.read_reg(0b111) & self.read_reg(src_reg);
                self.set_reg(0b111, result);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved

                self.set_szp(result);
                self.carry = false;
                self.auxc = true;
            }
            0xA8..=0xAF => {
                // XRA instructions
                let src_reg: u8 = instruction & 0b0000_0111;

                let result: u8 = self.read_reg(0b111) ^ self.read_reg(src_reg);
                self.set_reg(0b111, result);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved

                self.set_szp(result);
                self.carry = false;
                self.auxc = false;
            }
            0xB0..=0xB7 => {
                // ORA instructions
                let src_reg: u8 = instruction & 0b0000_0111;

                let result: u8 = self.read_reg(0b111) | self.read_reg(src_reg);
                self.set_reg(0b111, result);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved

                self.set_szp(result);
                self.carry = false;
                self.auxc = false;
            }
            0xB8..=0xBF => {
                // CMP instructions
                // just updates flags after subraction, A remains untouched
                let src_reg: u8 = instruction & 0b0000_0111;

                let a: u8 = self.read_reg(0b111);
                let b: u8 = self.read_reg(src_reg);
                let result: u8 = a.wrapping_sub(b);

                // TODO: figure out waiting for clock cycles
                // 4 for no M, 7 with M involved

                self.set_szp(result);
                self.carry = a < b;
                self.auxc = (a & 0x0F) < (b & 0x0F);
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                // POP instructions
                let lower: u8 = self.ram.read(self.sp);
                self.sp += 1;
                let upper: u8 = self.ram.read(self.sp);
                self.sp += 1;
                let word: u16 = ((upper as u16) << 8) | (lower as u16);

                let pair: u8 = (instruction & 0b00110000) >> 4;

                match pair {
                    0b00 => {
                        // pair BC
                        self.set_bc(word);
                    }
                    0b01 => {
                        // pair DE
                        self.set_de(word);
                    }
                    0b10 => {
                        // pair HL
                        self.set_hl(word);
                    }
                    0b11 => {
                        // PSW (A, flags)
                        self.set_psw(word);
                    }
                    _ => {/* invalid pair, shouldn't be possible */}
                }
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                // PUSH instructions
                let pair: u8 = (instruction & 0b00110000) >> 4;
                let mut word: u16 = 0;

                match pair {
                    0b00 => {
                        // pair BC
                        word = self.read_bc();
                    }
                    0b01 => {
                       // pair DE
                       word = self.read_de();
                    }
                    0b10 => {
                        // pair HL
                       word = self.read_hl();
                    }
                    0b11 => {
                        // PSW (A, flags)
                        word = self.read_psw();
                    }
                    _ => {/* invalid pair, shouldn't be possible */}
                }

                let upper: u8 = ((word & 0xFF00) >> 8) as u8;
                let lower: u8 = (word & 0x00FF) as u8;
                self.sp -= 1;
                self.ram.write(self.sp, upper);
                self.sp -= 1;
                self.ram.write(self.sp, lower);
            }
            0xF3 => {
                // DI (4 cycles)
                self.interrput_enable = false;
            }
            0xFB => {
                // EI (4 cycles)
                self.interrput_enable = true;
            }
            _ => {} 
        }
        self.pc = next_pc;
    }

}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "----------------------------------\r")?;
        writeln!(f, "CPU State:\r")?;
        writeln!(f, "  PC: 0x{:04X}\r", self.pc)?;
        writeln!(f, "  Registers:\r")?;
        writeln!(f, "    A: 0x{:02X}\r", self.registers[0b111 as usize ^ 1])?;
        writeln!(f, "    B: 0x{:02X}  C: 0x{:02X}\r", self.registers[0b000 as usize ^ 1], self.registers[0b001 as usize ^ 1])?;
        writeln!(f, "    D: 0x{:02X}  E: 0x{:02X}\r", self.registers[0b010 as usize ^ 1], self.registers[0b011 as usize ^ 1])?;
        writeln!(f, "    H: 0x{:02X}  L: 0x{:02X}\r", self.registers[0b100 as usize ^ 1], self.registers[0b101 as usize ^ 1])?;
        writeln!(f, "    FLAGS: {}\r", self.format_flags())?;
        writeln!(f, "----------------------------------\r")
    }
}