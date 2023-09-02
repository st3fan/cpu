
use super::*;

#[test]
fn cpu_read_instructions() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA9; // LDA #$42
    cpu.mem[0x0401] = 0x42;
    cpu.mem[0x0402] = 0x20; // JSR 0x1122
    cpu.mem[0x0403] = 0x22;
    cpu.mem[0x0404] = 0x11;
    cpu.pc = 0x0400;
    assert_eq!(0xA9, cpu.read_byte());
    assert_eq!(0x0401, cpu.pc);
    assert_eq!(0x42, cpu.read_byte());
    assert_eq!(0x0402, cpu.pc);
    assert_eq!(0x20, cpu.read_byte());
    assert_eq!(0x0403, cpu.pc);
    assert_eq!(0x1122, cpu.read_word());
    assert_eq!(0x0405, cpu.pc);
}

#[test]
fn load_and_store() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA9; // LDA #$42
    cpu.mem[0x0401] = 0x42;
    cpu.mem[0x0402] = 0x85; // STA $07
    cpu.mem[0x0403] = 0x07;
    cpu.mem[0x0405] = 0xFF; // So we exit with CPUError::IllegalInstruction
    assert_eq!(cpu.run(), Err(CPUError::IllegalInstruction));
    assert_eq!(0x42, cpu.a);
    assert_eq!(0x42, cpu.mem[0x0007]);
}

#[test]
fn push_pop_byte() {
    let mut cpu = CPU::new();

    cpu.push_byte(0x11);
    assert_eq!(0x11, cpu.mem[0x01ff]);
    assert_eq!(0xfe, cpu.s);

    let b = cpu.pop_byte();
    assert_eq!(0x11, b);
    assert_eq!(0xff, cpu.s);
    assert_eq!(0x11, cpu.mem[0x01ff]);
}

#[test]
fn push_pop_word() {
    let mut cpu = CPU::new();
    cpu.push_word(0x1234);
    assert_eq!(0x12, cpu.mem[0x01ff]);
    assert_eq!(0x34, cpu.mem[0x01fe]);
    assert_eq!(0xfd, cpu.s);
    let w = cpu.pop_word();
    assert_eq!(0x1234, w);
    assert_eq!(0xff, cpu.s);
}

#[test]
fn call_and_return() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xEA; // NOP
    cpu.mem[0x0401] = 0x20; // JSR 0x0405
    cpu.mem[0x0402] = 0x05;
    cpu.mem[0x0403] = 0x04;
    cpu.mem[0x0404] = 0xFF; // So we exit with CPUError::IllegalInstruction
    cpu.mem[0x0405] = 0xA2; // LDX #$65
    cpu.mem[0x0406] = 0x65;
    cpu.mem[0x0407] = 0x86; // STX $05
    cpu.mem[0x0408] = 0x05;
    cpu.mem[0x0409] = 0xA2; // LDX #$02
    cpu.mem[0x040A] = 0x02;
    cpu.mem[0x040B] = 0x86; // STX $06
    cpu.mem[0x040C] = 0x06;
    cpu.mem[0x040D] = 0x60; // RTS
    assert_eq!(cpu.run(), Err(CPUError::IllegalInstruction));
    assert_eq!(0x65, cpu.mem[0x0005]);
    assert_eq!(0x02, cpu.mem[0x0006]);
}

#[test]
fn n_is_set() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA9; // LDA #$80
    cpu.mem[0x0401] = 0x80;
    cpu.mem[0x0402] = 0xFF; // So we exit with CPUError::IllegalInstruction
    assert_eq!(cpu.run(), Err(CPUError::IllegalInstruction));
    assert!(cpu.p.contains(Status::N));
    cpu.mem[0x0403] = 0xA9; // LDA #$7F
    cpu.mem[0x0404] = 0x7F;
    cpu.mem[0x0405] = 0xFF; // So we exit with CPUError::IllegalInstruction
    assert_eq!(cpu.run(), Err(CPUError::IllegalInstruction));
    assert!(!cpu.p.contains(Status::N));
}

#[test]
fn z_is_set() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA9; // LDA #$00
    cpu.mem[0x0401] = 0x00;
    cpu.mem[0x0402] = 0xFF; // So we exit with CPUError::IllegalInstruction
    assert_eq!(cpu.run(), Err(CPUError::IllegalInstruction));
    assert!(cpu.p.contains(Status::Z));
    cpu.mem[0x0403] = 0xA9; // LDA #$01
    cpu.mem[0x0404] = 0x01;
    cpu.mem[0x0405] = 0xFF; // So we exit with CPUError::IllegalInstruction
    assert_eq!(cpu.run(), Err(CPUError::IllegalInstruction));
    assert!(!cpu.p.contains(Status::N));
}