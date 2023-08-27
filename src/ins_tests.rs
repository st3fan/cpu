use super::*;

#[test]
fn lda_imm() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA9; // LDX #$42
    cpu.mem[0x0401] = 0x42;
    cpu.mem[0x0402] = 0x00;
    cpu.run();
    assert_eq!(0x42, cpu.a);
}

#[test]
fn sta_zp() {
    let mut cpu = CPU::new();
    cpu.a = 0x42;
    cpu.mem[0x0400] = 0x85; // STA $07
    cpu.mem[0x0401] = 0x07;
    cpu.mem[0x0402] = 0x00;
    cpu.run();
    assert_eq!(0x42, cpu.mem[0x07]);
}

#[test]
fn ldx_imm() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA2; // LDX #$65
    cpu.mem[0x0401] = 0x65;
    cpu.mem[0x0402] = 0x00;
    cpu.run();
    assert_eq!(0x65, cpu.x);
}

#[test]
fn stx_zp() {
    let mut cpu = CPU::new();
    cpu.x = 0x42;
    cpu.mem[0x0400] = 0x86; // STX $07
    cpu.mem[0x0401] = 0x07;
    cpu.mem[0x0402] = 0x00;
    cpu.run();
    assert_eq!(0x42, cpu.mem[0x07]);
}
