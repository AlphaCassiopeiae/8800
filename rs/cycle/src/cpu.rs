
#[derive(Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    flags: Flags,
}

#[derive(Default)]
struct Flags {
    sign: bool,
    zero: bool,
    aux_carry: bool,
    parity: bool,
    carry: bool,
}



fn get mem() // write to memory on the s100 bus


fn write mem() // write to memory on the s100 bus
