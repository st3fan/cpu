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
    cpu.run();
    assert_eq!(0x42, cpu.mem[0x07]);
}

#[test]
fn ldx_imm() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA2; // LDX #$65
    cpu.mem[0x0401] = 0x65;
    cpu.run();
    assert_eq!(0x65, cpu.x);
}

#[test]
fn stx_zp() {
    let mut cpu = CPU::new();
    cpu.x = 0x42;
    cpu.mem[0x0400] = 0x86; // STX $07
    cpu.mem[0x0401] = 0x07;
    cpu.run();
    assert_eq!(0x42, cpu.mem[0x07]);
}

#[test]
fn test_dex() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA2; // LDX #$12
    cpu.mem[0x0401] = 0x12;
    cpu.mem[0x0402] = 0xCA; // DEX
    cpu.run();
    assert_eq!(0x11, cpu.x);
    assert!(cpu.p.is_empty());    
}

#[test]
fn test_dex_z() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA2; // LDX #$12
    cpu.mem[0x0401] = 0x01;
    cpu.mem[0x0402] = 0xCA; // DEX
    cpu.run();
    assert_eq!(0x00, cpu.x);
    assert_eq!(cpu.p.contains(Status::Z), true);    
    assert_eq!(cpu.p.contains(Status::N), false);
}

#[test]
fn test_dex_n() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA2; // LDX #$12
    cpu.mem[0x0401] = 0x88;
    cpu.mem[0x0402] = 0xCA; // DEX
    cpu.run();
    assert_eq!(0x87, cpu.x);
    assert_eq!(cpu.p.contains(Status::Z), false);
    assert_eq!(cpu.p.contains(Status::N), true);
}

#[test]
fn test_dey() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA0; // LDY #$12
    cpu.mem[0x0401] = 0x12;
    cpu.mem[0x0402] = 0x88; // DEY
    cpu.run();
    assert_eq!(0x11, cpu.y);
    assert!(cpu.p.is_empty());
}

#[test]
fn test_dey_z() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA0; // LDY #$12
    cpu.mem[0x0401] = 0x01;
    cpu.mem[0x0402] = 0x88; // DEX
    cpu.run();
    assert_eq!(0x00, cpu.y);
    assert_eq!(cpu.p.contains(Status::Z), true);    
    assert_eq!(cpu.p.contains(Status::N), false);
}

#[test]
fn test_dey_n() {
    let mut cpu = CPU::new();
    cpu.mem[0x0400] = 0xA0; // LDY #$12
    cpu.mem[0x0401] = 0x88;
    cpu.mem[0x0402] = 0x88; // DEY
    cpu.run();
    assert_eq!(0x87, cpu.y);
    assert_eq!(cpu.p.contains(Status::Z), false);
    assert_eq!(cpu.p.contains(Status::N), true);
}

// #[test]
// fn casting_u8_to_i16() {
//     let a: u8 = 0xFE; // -2
//     let b: i16 = a as i8 as i16;
//     assert_eq!(b, -2);
//     let mut pc: u16 = 0x0400;
//     let t = pc as i16;
//     pc = t.wrapping_add(b) as u16;
//     assert_eq!(pc, 0x03FE);
// }
