use core::prelude::rust_2024::derive;
use core::marker::Copy;
use core::clone::Clone;
use core::fmt::Debug;

const MEM_SIZE: usize = 65536;

#[derive(Debug, Clone, Copy)]
pub struct RAM {
    bytes: [u8; MEM_SIZE],
}

impl RAM {
    pub fn new() -> Self {
        Self {
            bytes: [0; MEM_SIZE],
        }
    }

    // for simulating instructions being present in memory, do nothing for now
    pub fn init(&mut self) {
        // self.bytes[0] = 0x15;
        // self.bytes[1] = 0xD3;
        // self.bytes[2] = 0x76;

        // for addr in 0..=255 {
        //     self.bytes[addr] = addr as u8;
        // }

        self.bytes[0] = 0x3C; // INR A, A = 0x01
        self.bytes[1] = 0x32; // STA a16 
        self.bytes[2] = 0xEF;
        self.bytes[3] = 0xBE; // addr = 0xBEEF   
        self.bytes[4] = 0x3C; // INR A, A = 0x02
        self.bytes[5] = 0x3A; // LDA a16
        self.bytes[6] = 0xEF;
        self.bytes[7] = 0xBE; // addr = 0xBEEF
        self.bytes[8] = 0x76; // HLT
        // expect A = 0x01, not 0x02
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.bytes[addr]
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        self.bytes[addr] = data;
    }
}