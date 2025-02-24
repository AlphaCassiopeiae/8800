mod recall;
use recall::RAM;

fn main() -> ! {
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

    // this is where we will be checking if memory control signals are asserted
    // respond accordingly (read or write)
    // A0-A15: address pins
    // D0-D7: data pins
    // nPWR: low-active, bus signal confirming D0-D7 is valid
    // MWRITE: identifies read operation
    // SMEMR: identifes read operation
    // PDBIN: control signal, tells memory to put read value on D0-D7
    loop {
        // comment this out when want to test
        panic!("not done with micro stuff yet");
    }
}
