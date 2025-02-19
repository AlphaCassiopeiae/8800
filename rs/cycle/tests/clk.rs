use my_project::cpu::CPU;  // Import the CPU struct from your project

#[test]
fn test_mvi_a() {
    let mut cpu = CPU::new();
    cpu.execute_instruction(0x3E);  // MVI A, 0x42
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn test_inr_b() {
    let mut cpu = CPU::new();
    cpu.b = 0x10;
    cpu.execute_instruction(0x04);  // INR B
    assert_eq!(cpu.b, 0x11);
}

// hi test
#[df]