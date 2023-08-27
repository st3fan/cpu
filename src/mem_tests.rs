use super::*;

fn new_test_cpu() -> CPU {
    let mut cpu = CPU::new();
    cpu.mem[0x0000] = 0x10;
    cpu.mem[0x0001] = 0x11;
    cpu.mem[0x0002] = 0x12;
    cpu.mem[0x0003] = 0x13;
    cpu.mem[0x0004] = 0x14;
    cpu.mem[0x0005] = 0x15;
    cpu.mem[0x0006] = 0x16;
    cpu.mem[0x0007] = 0x17;

    cpu.mem[0x0010] = 0x00;
    cpu.mem[0x0011] = 0x00;
    cpu.mem[0x0012] = 0x00;
    cpu.mem[0x0013] = 0x0a;
    cpu.mem[0x0014] = 0x04;

    cpu.mem[0x00ff] = 0x2f;

    cpu.mem[0x0400] = 0xEA; // NOP
    cpu.mem[0x0401] = 0x20; // JSR 0x0405
    cpu.mem[0x0402] = 0x04;
    cpu.mem[0x0403] = 0x05;
    cpu.mem[0x0004] = 0x00; // BRK
    cpu.mem[0x0405] = 0xA2; // LDX #$65
    cpu.mem[0x0406] = 0x65;
    cpu.mem[0x0407] = 0x86; // STX $05
    cpu.mem[0x0408] = 0x05;
    cpu.mem[0x0409] = 0xA2; // LDX #$02
    cpu.mem[0x040A] = 0x02;
    cpu.mem[0x040B] = 0x86; // STX $06
    cpu.mem[0x040C] = 0x06;
    cpu.mem[0x040D] = 0x60; // RTS
    cpu
}

// Memory Accessors

#[test]
fn cpu_get_byte_zpg() {
    let mut cpu = new_test_cpu();
    assert_eq!(0x15, cpu.get_byte_zpg(0x05));
}

#[test]
fn cpu_get_byte_zpgx() {
    let mut cpu = new_test_cpu();
    cpu.x = 0x00;
    assert_eq!(0x10, cpu.get_byte_zpgx(0x00));
    cpu.x = 0x03;
    assert_eq!(0x13, cpu.get_byte_zpgx(0x00));
    cpu.x = 0x00;
    assert_eq!(0x2f, cpu.get_byte_zpgx(0xff));
    cpu.x = 0x03;
    assert_eq!(0x12, cpu.get_byte_zpgx(0xff));
    cpu.x = 0xff;
    assert_eq!(0x2f, cpu.get_byte_zpgx(0x00));
}

#[test]
fn cpu_get_byte_zpgy() {
    let mut cpu = new_test_cpu();
    cpu.y = 0x00;
    assert_eq!(0x10, cpu.get_byte_zpgy(0x00));
    cpu.y = 0x03;
    assert_eq!(0x13, cpu.get_byte_zpgy(0x00));
    cpu.y = 0x00;
    assert_eq!(0x2f, cpu.get_byte_zpgy(0xff));
    cpu.y = 0x03;
    assert_eq!(0x12, cpu.get_byte_zpgy(0xff));
    cpu.y = 0xff;
    assert_eq!(0x2f, cpu.get_byte_zpgy(0x00));
}

#[test]
fn cpu_get_byte_abs() {
    let mut cpu = new_test_cpu();
    assert_eq!(0xEA, cpu.get_byte_abs(0x0400));
}

#[test]
fn cpu_get_byte_absx() {
    let mut cpu = new_test_cpu();
    cpu.x = 0x03;
    assert_eq!(0x05, cpu.get_byte_absx(0x0400));        
}

#[test]
fn cpu_get_byte_absy() {
    let mut cpu = new_test_cpu();
    cpu.y = 0x03;
    assert_eq!(0x05, cpu.get_byte_absy(0x0400));        
}

#[test]
fn cpu_get_byte_xind() {
    let mut cpu = new_test_cpu();
    cpu.x = 3;
    assert_eq!(0x02, cpu.get_byte_xind(0x10));
}

#[test]
fn cpu_get_byte_indy() {
    let mut cpu = new_test_cpu();        
    cpu.y = 3;
    assert_eq!(0x60, cpu.get_byte_indy(0x13));
}

// Setters

#[test]
fn set_byte() {
    let mut cpu = new_test_cpu();
    cpu.set_byte(0x0400, 0x12);
    assert_eq!(0x12, cpu.mem[0x0400]);
}

#[test]
fn cpu_set_byte_zpg() {
    let mut cpu = new_test_cpu();
    cpu.set_byte_zpg(0x42, 0x21);
    assert_eq!(0x21, cpu.mem[0x0042]);
}

#[test]
fn cpu_set_byte_zpgx() {
    let mut cpu = new_test_cpu();
    cpu.x = 0;
    cpu.set_byte_zpgx(0x42, 0x11);
    assert_eq!(0x11, cpu.mem[0x0042]);
    cpu.x = 1;
    cpu.set_byte_zpgx(0x42, 0x22);
    assert_eq!(0x22, cpu.mem[0x0043]);
    cpu.x = 1;
    cpu.set_byte_zpgx(0xff, 0x33);
    assert_eq!(0x33, cpu.mem[0x0000]);
    cpu.x = 3;
    cpu.set_byte_zpgx(0xff, 0x44);
    assert_eq!(0x44, cpu.mem[0x0002]);
}

#[test]
fn cpu_set_byte_zpgy() {
    let mut cpu = new_test_cpu();
    cpu.y = 0;
    cpu.set_byte_zpgy(0x42, 0x11);
    assert_eq!(0x11, cpu.mem[0x0042]);
    cpu.y = 1;
    cpu.set_byte_zpgy(0x42, 0x22);
    assert_eq!(0x22, cpu.mem[0x0043]);
    cpu.y = 1;
    cpu.set_byte_zpgy(0xff, 0x33);
    assert_eq!(0x33, cpu.mem[0x0000]);
    cpu.y = 3;
    cpu.set_byte_zpgy(0xff, 0x44);
    assert_eq!(0x44, cpu.mem[0x0002]);
}

#[test]
fn cpu_set_byte_abs() {
    let mut cpu = new_test_cpu();
    cpu.set_byte_abs(0x0400, 0x12);
    assert_eq!(0x12, cpu.mem[0x0400]);
}

#[test]
fn cpu_set_byte_absx() {
    let mut cpu = new_test_cpu();
    cpu.x = 3;
    cpu.set_byte_absx(0x0400, 0x12);
    assert_eq!(0x12, cpu.mem[0x0403]);
}

#[test]
fn cpu_set_byte_absy() {
    let mut cpu = new_test_cpu();
    cpu.y = 3;
    cpu.set_byte_absy(0x0400, 0x12);
    assert_eq!(0x12, cpu.mem[0x0403]);
}

#[test]
fn cpu_set_byte_xind() {
    let mut cpu = new_test_cpu();
    cpu.x = 3;
    cpu.set_byte_xind(0x10, 0x42);
    assert_eq!(0x42, cpu.get_byte_xind(0x10)); // TODO Ok to depend on get_byte_xind?
}

#[test]
fn cpu_set_byte_indy() {
    let mut cpu = new_test_cpu();
    cpu.y = 3;
    cpu.set_byte_indy(0x13, 0x42);
    assert_eq!(0x42, cpu.get_byte_indy(0x13));
}