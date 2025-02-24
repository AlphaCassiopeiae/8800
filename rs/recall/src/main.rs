mod recall;
use recall::RAM;

fn main() {
    let mut memory: RAM = RAM::new();

    // testing basic read/write
    let byte: u8 = 100;
    let addr: usize = 10;
    memory.write(addr, byte);

    let mut readval: u8 = memory.read(addr);
    println!("The value {} is at address {}", readval, addr);

    // reading random address should return 0
    let unset_addr: usize = 15;
    readval =  memory.read(unset_addr);
    println!("The value {} is at address {}", readval, unset_addr);
}
