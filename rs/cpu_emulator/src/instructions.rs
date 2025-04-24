// instructions.rs - defines instruction lookup table
use defmt::*;
use core::prelude::rust_2024::derive;
use core::marker::Copy;
use core::clone::Clone;
use core::fmt::Debug;
use core::fmt;
use core::writeln;

#[derive(Debug, Clone, Copy, Format)]
pub struct InstructionInfo {
    pub mnemonic: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub affected_flags: u8, // 1 -> flag is affected, 0 -> flag noe affected
}

impl fmt::Display for InstructionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}, bytes: {}, cycles: {}, affected_flags: 0b{:08b}\r", self.mnemonic, self.bytes, self.cycles, self.affected_flags)
    }
}

pub static INSTRUCTION_TABLE: [InstructionInfo; 256] = {
    let mut table: [InstructionInfo; 256] = [InstructionInfo {
        mnemonic: "INVALID",
        bytes: 1,
        cycles: 0,
        affected_flags: 0x00,
    }; 256];

    table[0x00] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x01] = InstructionInfo{mnemonic: "LXI B,d16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0x02] = InstructionInfo{mnemonic: "STAX B", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x03] = InstructionInfo{mnemonic: "INX B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x04] = InstructionInfo{mnemonic: "INR B", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x05] = InstructionInfo{mnemonic: "DCR B", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x06] = InstructionInfo{mnemonic: "MVI B,d8", bytes: 2, cycles: 7, affected_flags: 0b0000_0000};
    table[0x07] = InstructionInfo{mnemonic: "RLC", bytes: 1, cycles: 4, affected_flags: 0b0000_0001};
    table[0x08] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x09] = InstructionInfo{mnemonic: "DAD B", bytes: 1, cycles: 10, affected_flags: 0b0000_0001};
    table[0x0A] = InstructionInfo{mnemonic: "LDAX B", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x0B] = InstructionInfo{mnemonic: "DCX B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x0C] = InstructionInfo{mnemonic: "INR C", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x0D] = InstructionInfo{mnemonic: "DCR C", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x0E] = InstructionInfo{mnemonic: "MVI C,d8", bytes: 2, cycles: 7, affected_flags: 0b0000_0000};
    table[0x0F] = InstructionInfo{mnemonic: "RRC", bytes: 1, cycles: 4, affected_flags: 0b0000_0001};
    //...
    table[0x10] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x11] = InstructionInfo{mnemonic: "LXI D,d16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0x12] = InstructionInfo{mnemonic: "STAX D", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x13] = InstructionInfo{mnemonic: "INX D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x14] = InstructionInfo{mnemonic: "INR D", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x15] = InstructionInfo{mnemonic: "DCR D", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x16] = InstructionInfo{mnemonic: "MVI D,d8", bytes: 2, cycles: 7, affected_flags: 0b0000_0000};
    table[0x17] = InstructionInfo{mnemonic: "RAL", bytes: 1, cycles: 4, affected_flags: 0b0000_0001};
    table[0x18] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x19] = InstructionInfo{mnemonic: "DAD D", bytes: 1, cycles: 10, affected_flags: 0b0000_0001};
    table[0x1A] = InstructionInfo{mnemonic: "LDAX D", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x1B] = InstructionInfo{mnemonic: "DCX D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x1C] = InstructionInfo{mnemonic: "INR E", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x1D] = InstructionInfo{mnemonic: "DCR E", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x1E] = InstructionInfo{mnemonic: "MVI E,d8", bytes: 2, cycles: 7, affected_flags: 0b0000_0000};
    table[0x1F] = InstructionInfo{mnemonic: "RAR", bytes: 1, cycles: 4, affected_flags: 0b0000_0001};

    table[0x20] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x21] = InstructionInfo{mnemonic: "LXI H,d16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0x22] = InstructionInfo{mnemonic: "SHLD a16", bytes: 3, cycles: 16, affected_flags: 0b0000_0000};
    table[0x23] = InstructionInfo{mnemonic: "INX H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x24] = InstructionInfo{mnemonic: "INR H", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x25] = InstructionInfo{mnemonic: "DCR H", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x26] = InstructionInfo{mnemonic: "MVI H,d8", bytes: 2, cycles: 7, affected_flags: 0b0000_0000};
    table[0x27] = InstructionInfo{mnemonic: "DAA", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x28] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x29] = InstructionInfo{mnemonic: "DAD H", bytes: 1, cycles: 10, affected_flags: 0b0000_0001};
    table[0x2A] = InstructionInfo{mnemonic: "LHLD a16", bytes: 3, cycles: 16, affected_flags: 0b0000_0000};
    table[0x2B] = InstructionInfo{mnemonic: "DCX H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x2C] = InstructionInfo{mnemonic: "INR L", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x2D] = InstructionInfo{mnemonic: "DCR L", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x2E] = InstructionInfo{mnemonic: "MVI L,d8", bytes: 2, cycles: 7, affected_flags: 0b0000_0000};
    table[0x2F] = InstructionInfo{mnemonic: "CMA", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};

    table[0x30] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x31] = InstructionInfo{mnemonic: "LXI SP,d16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0x32] = InstructionInfo{mnemonic: "STA a16", bytes: 3, cycles: 13, affected_flags: 0b0000_0000};
    table[0x33] = InstructionInfo{mnemonic: "INX SP", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x34] = InstructionInfo{mnemonic: "INR M", bytes: 1, cycles: 10, affected_flags: 0b1101_0100};
    table[0x35] = InstructionInfo{mnemonic: "DCR M", bytes: 1, cycles: 10, affected_flags: 0b1101_0100};
    table[0x36] = InstructionInfo{mnemonic: "MVI M,d8", bytes: 2, cycles: 10, affected_flags: 0b0000_0000};
    table[0x37] = InstructionInfo{mnemonic: "STC", bytes: 1, cycles: 4, affected_flags: 0b0000_0001};
    table[0x38] = InstructionInfo{mnemonic: "NOP", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0x39] = InstructionInfo{mnemonic: "DAD SP", bytes: 1, cycles: 10, affected_flags: 0b0000_0001};
    table[0x3A] = InstructionInfo{mnemonic: "LDA a16", bytes: 3, cycles: 13, affected_flags: 0b0000_0000};
    table[0x3B] = InstructionInfo{mnemonic: "DCX SP", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x3C] = InstructionInfo{mnemonic: "INR A", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x3D] = InstructionInfo{mnemonic: "DCR A", bytes: 1, cycles: 5, affected_flags: 0b1101_0100};
    table[0x3E] = InstructionInfo{mnemonic: "MVI A,d8", bytes: 2, cycles: 7, affected_flags: 0b0000_0000};
    table[0x3F] = InstructionInfo{mnemonic: "CMC", bytes: 1, cycles: 4, affected_flags: 0b0000_0001};

    table[0x40] = InstructionInfo{mnemonic: "MOV B,B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x41] = InstructionInfo{mnemonic: "MOV B,C", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x42] = InstructionInfo{mnemonic: "MOV B,D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x43] = InstructionInfo{mnemonic: "MOV B,E", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x44] = InstructionInfo{mnemonic: "MOV B,H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x45] = InstructionInfo{mnemonic: "MOV B,L", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x46] = InstructionInfo{mnemonic: "MOV B,M", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x47] = InstructionInfo{mnemonic: "MOV B,A", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x48] = InstructionInfo{mnemonic: "MOV C,B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x49] = InstructionInfo{mnemonic: "MOV C,C", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x4A] = InstructionInfo{mnemonic: "MOV C,D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x4B] = InstructionInfo{mnemonic: "MOV C,E", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x4C] = InstructionInfo{mnemonic: "MOV C,H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x4D] = InstructionInfo{mnemonic: "MOV C,L", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x4E] = InstructionInfo{mnemonic: "MOV C,M", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x4F] = InstructionInfo{mnemonic: "MOV C,A", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};

    table[0x50] = InstructionInfo{mnemonic: "MOV D,B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x51] = InstructionInfo{mnemonic: "MOV D,C", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x52] = InstructionInfo{mnemonic: "MOV D,D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x53] = InstructionInfo{mnemonic: "MOV D,E", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x54] = InstructionInfo{mnemonic: "MOV D,H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x55] = InstructionInfo{mnemonic: "MOV D,L", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x56] = InstructionInfo{mnemonic: "MOV D,M", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x57] = InstructionInfo{mnemonic: "MOV D,A", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x58] = InstructionInfo{mnemonic: "MOV E,B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x59] = InstructionInfo{mnemonic: "MOV E,C", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x5A] = InstructionInfo{mnemonic: "MOV E,D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x5B] = InstructionInfo{mnemonic: "MOV E,E", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x5C] = InstructionInfo{mnemonic: "MOV E,H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x5D] = InstructionInfo{mnemonic: "MOV E,L", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x5E] = InstructionInfo{mnemonic: "MOV E,M", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x5F] = InstructionInfo{mnemonic: "MOV E,A", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};

    table[0x60] = InstructionInfo{mnemonic: "MOV H,B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x61] = InstructionInfo{mnemonic: "MOV H,C", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x62] = InstructionInfo{mnemonic: "MOV H,D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x63] = InstructionInfo{mnemonic: "MOV H,E", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x64] = InstructionInfo{mnemonic: "MOV H,H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x65] = InstructionInfo{mnemonic: "MOV H,L", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x66] = InstructionInfo{mnemonic: "MOV H,M", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x67] = InstructionInfo{mnemonic: "MOV H,A", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x68] = InstructionInfo{mnemonic: "MOV L,B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x69] = InstructionInfo{mnemonic: "MOV L,C", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x6A] = InstructionInfo{mnemonic: "MOV L,D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x6B] = InstructionInfo{mnemonic: "MOV L,E", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x6C] = InstructionInfo{mnemonic: "MOV L,H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x6D] = InstructionInfo{mnemonic: "MOV L,L", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x6E] = InstructionInfo{mnemonic: "MOV L,M", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x6F] = InstructionInfo{mnemonic: "MOV L,A", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};

    table[0x70] = InstructionInfo{mnemonic: "MOV M,B", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x71] = InstructionInfo{mnemonic: "MOV M,C", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x72] = InstructionInfo{mnemonic: "MOV M,D", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x73] = InstructionInfo{mnemonic: "MOV M,E", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x74] = InstructionInfo{mnemonic: "MOV M,H", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x75] = InstructionInfo{mnemonic: "MOV M,L", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x76] = InstructionInfo{mnemonic: "HLT",     bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x77] = InstructionInfo{mnemonic: "MOV M,A", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x78] = InstructionInfo{mnemonic: "MOV A,B", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x79] = InstructionInfo{mnemonic: "MOV A,C", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x7A] = InstructionInfo{mnemonic: "MOV A,D", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x7B] = InstructionInfo{mnemonic: "MOV A,E", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x7C] = InstructionInfo{mnemonic: "MOV A,H", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x7D] = InstructionInfo{mnemonic: "MOV A,L", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0x7E] = InstructionInfo{mnemonic: "MOV A,M", bytes: 1, cycles: 7, affected_flags: 0b0000_0000};
    table[0x7F] = InstructionInfo{mnemonic: "MOV A,A", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};

    table[0x80] = InstructionInfo{mnemonic: "ADD B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x81] = InstructionInfo{mnemonic: "ADD C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x82] = InstructionInfo{mnemonic: "ADD D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x83] = InstructionInfo{mnemonic: "ADD E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x84] = InstructionInfo{mnemonic: "ADD H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x85] = InstructionInfo{mnemonic: "ADD L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x86] = InstructionInfo{mnemonic: "ADD M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0x87] = InstructionInfo{mnemonic: "ADD A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x88] = InstructionInfo{mnemonic: "ADC B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x89] = InstructionInfo{mnemonic: "ADC C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x8A] = InstructionInfo{mnemonic: "ADC D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x8B] = InstructionInfo{mnemonic: "ADC E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x8C] = InstructionInfo{mnemonic: "ADC H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x8D] = InstructionInfo{mnemonic: "ADC L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x8E] = InstructionInfo{mnemonic: "ADC M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0x8F] = InstructionInfo{mnemonic: "ADC A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};

    table[0x90] = InstructionInfo{mnemonic: "SUB B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x91] = InstructionInfo{mnemonic: "SUB C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x92] = InstructionInfo{mnemonic: "SUB D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x93] = InstructionInfo{mnemonic: "SUB E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x94] = InstructionInfo{mnemonic: "SUB H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x95] = InstructionInfo{mnemonic: "SUB L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x96] = InstructionInfo{mnemonic: "SUB M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0x97] = InstructionInfo{mnemonic: "SUB A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x98] = InstructionInfo{mnemonic: "SBB B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x99] = InstructionInfo{mnemonic: "SBB C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x9A] = InstructionInfo{mnemonic: "SBB D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x9B] = InstructionInfo{mnemonic: "SBB E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x9C] = InstructionInfo{mnemonic: "SBB H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x9D] = InstructionInfo{mnemonic: "SBB L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0x9E] = InstructionInfo{mnemonic: "SBB M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0x9F] = InstructionInfo{mnemonic: "SBB A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};

    table[0xA0] = InstructionInfo{mnemonic: "ANA B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA1] = InstructionInfo{mnemonic: "ANA C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA2] = InstructionInfo{mnemonic: "ANA D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA3] = InstructionInfo{mnemonic: "ANA E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA4] = InstructionInfo{mnemonic: "ANA H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA5] = InstructionInfo{mnemonic: "ANA L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA6] = InstructionInfo{mnemonic: "ANA M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0xA7] = InstructionInfo{mnemonic: "ANA A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA8] = InstructionInfo{mnemonic: "XRA B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xA9] = InstructionInfo{mnemonic: "XRA C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xAA] = InstructionInfo{mnemonic: "XRA D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xAB] = InstructionInfo{mnemonic: "XRA E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xAC] = InstructionInfo{mnemonic: "XRA H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xAD] = InstructionInfo{mnemonic: "XRA L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xAE] = InstructionInfo{mnemonic: "XRA M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0xAF] = InstructionInfo{mnemonic: "XRA A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};

    table[0xB0] = InstructionInfo{mnemonic: "ORA B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB1] = InstructionInfo{mnemonic: "ORA C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB2] = InstructionInfo{mnemonic: "ORA D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB3] = InstructionInfo{mnemonic: "ORA E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB4] = InstructionInfo{mnemonic: "ORA H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB5] = InstructionInfo{mnemonic: "ORA L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB6] = InstructionInfo{mnemonic: "ORA M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0xB7] = InstructionInfo{mnemonic: "ORA A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB8] = InstructionInfo{mnemonic: "CMP B", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xB9] = InstructionInfo{mnemonic: "CMP C", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xBA] = InstructionInfo{mnemonic: "CMP D", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xBB] = InstructionInfo{mnemonic: "CMP E", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xBC] = InstructionInfo{mnemonic: "CMP H", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xBD] = InstructionInfo{mnemonic: "CMP L", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};
    table[0xBE] = InstructionInfo{mnemonic: "CMP M", bytes: 1, cycles: 7, affected_flags: 0b1101_0101};
    table[0xBF] = InstructionInfo{mnemonic: "CMP A", bytes: 1, cycles: 4, affected_flags: 0b1101_0101};

    // need to figure out how to handle those that can take different number of cycles
    // check outside of table, in case statement later
    // if conditional not taken, change cycles (assume taken in lookup)
    table[0xC0] = InstructionInfo{mnemonic: "RNZ", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xC1] = InstructionInfo{mnemonic: "POP B", bytes: 1, cycles: 10, affected_flags: 0b0000_0000};
    table[0xC2] = InstructionInfo{mnemonic: "JNZ a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xC3] = InstructionInfo{mnemonic: "JMP a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xC4] = InstructionInfo{mnemonic: "CNZ a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xC5] = InstructionInfo{mnemonic: "PUSH B", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xC6] = InstructionInfo{mnemonic: "ADI d8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xC7] = InstructionInfo{mnemonic: "RST 0", bytes: 1, cycles: 11, affected_flags: 00000_0000};
    table[0xC8] = InstructionInfo{mnemonic: "RZ", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xC9] = InstructionInfo{mnemonic: "RET", bytes: 1, cycles: 10, affected_flags: 0b0000_0000};
    table[0xCA] = InstructionInfo{mnemonic: "JZ a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xCB] = InstructionInfo{mnemonic: "JMP a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xCC] = InstructionInfo{mnemonic: "CZ a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xCD] = InstructionInfo{mnemonic: "CALL a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xCE] = InstructionInfo{mnemonic: "ACI d8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xCF] = InstructionInfo{mnemonic: "RST 1", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};

    table[0xD0] = InstructionInfo{mnemonic: "RNC", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xD1] = InstructionInfo{mnemonic: "POP D", bytes: 1, cycles: 10, affected_flags: 0b0000_0000};
    table[0xD2] = InstructionInfo{mnemonic: "JNC a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xD3] = InstructionInfo{mnemonic: "OUT d8", bytes: 2, cycles: 10, affected_flags: 0b0000_0000};
    table[0xD4] = InstructionInfo{mnemonic: "CNC a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xD5] = InstructionInfo{mnemonic: "PUSH D", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xD6] = InstructionInfo{mnemonic: "SUI d8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xD7] = InstructionInfo{mnemonic: "RST 2", bytes: 1, cycles: 11, affected_flags: 00000_0000};
    table[0xD8] = InstructionInfo{mnemonic: "RC", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xD9] = InstructionInfo{mnemonic: "RET", bytes: 1, cycles: 10, affected_flags: 0b0000_0000};
    table[0xDA] = InstructionInfo{mnemonic: "JC a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xDB] = InstructionInfo{mnemonic: "IN d8", bytes: 2, cycles: 10, affected_flags: 0b0000_0000};
    table[0xDC] = InstructionInfo{mnemonic: "CC a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xDD] = InstructionInfo{mnemonic: "CALL a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xDE] = InstructionInfo{mnemonic: "SBI d8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xDF] = InstructionInfo{mnemonic: "RST 3", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};

    table[0xE0] = InstructionInfo{mnemonic: "RPO", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xE1] = InstructionInfo{mnemonic: "POP H", bytes: 1, cycles: 10, affected_flags: 0b0000_0000};
    table[0xE2] = InstructionInfo{mnemonic: "JPO a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xE3] = InstructionInfo{mnemonic: "XTHL", bytes: 1, cycles: 18, affected_flags: 0b0000_0000};
    table[0xE4] = InstructionInfo{mnemonic: "CPO a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xE5] = InstructionInfo{mnemonic: "PUSH H", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xE6] = InstructionInfo{mnemonic: "ANI d8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xE7] = InstructionInfo{mnemonic: "RST 4", bytes: 1, cycles: 11, affected_flags: 00000_0000};
    table[0xE8] = InstructionInfo{mnemonic: "RPE", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xE9] = InstructionInfo{mnemonic: "PCHL", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0xEA] = InstructionInfo{mnemonic: "JPE a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xEB] = InstructionInfo{mnemonic: "XCHG", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0xEC] = InstructionInfo{mnemonic: "CPE a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xED] = InstructionInfo{mnemonic: "CALL a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xEE] = InstructionInfo{mnemonic: "XRI d8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xEF] = InstructionInfo{mnemonic: "RST 5", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};

    table[0xF0] = InstructionInfo{mnemonic: "RP", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xF1] = InstructionInfo{mnemonic: "POP PSW", bytes: 1, cycles: 10, affected_flags: 0b1101_0101};
    table[0xF2] = InstructionInfo{mnemonic: "JP a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xF3] = InstructionInfo{mnemonic: "DI", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0xF4] = InstructionInfo{mnemonic: "CP a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xF5] = InstructionInfo{mnemonic: "PUSH PSW", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xF6] = InstructionInfo{mnemonic: "ORId8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xF7] = InstructionInfo{mnemonic: "RST 6", bytes: 1, cycles: 11, affected_flags: 00000_0000};
    table[0xF8] = InstructionInfo{mnemonic: "RM", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};
    table[0xF9] = InstructionInfo{mnemonic: "SPHL", bytes: 1, cycles: 5, affected_flags: 0b0000_0000};
    table[0xFA] = InstructionInfo{mnemonic: "JM a16", bytes: 3, cycles: 10, affected_flags: 0b0000_0000};
    table[0xFB] = InstructionInfo{mnemonic: "EI", bytes: 1, cycles: 4, affected_flags: 0b0000_0000};
    table[0xFC] = InstructionInfo{mnemonic: "CM a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xFD] = InstructionInfo{mnemonic: "CALL a16", bytes: 3, cycles: 17, affected_flags: 0b0000_0000};
    table[0xFE] = InstructionInfo{mnemonic: "CPI d8", bytes: 2, cycles: 7, affected_flags: 0b1101_0101};
    table[0xFF] = InstructionInfo{mnemonic: "RST 7", bytes: 1, cycles: 11, affected_flags: 0b0000_0000};

    table
};