// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/

use bitflags::bitflags;

//
// This is a 6502 emulator with the following memory layout:
//
//  0x0000 - 0x00ff RAM Zero Page
//  0x0100 - 0x01ff RAM Stack
//  0x0200 - 0x03ff RAM General Use
//  0x0400 - 0x07ff ROM
//

bitflags! {
    pub struct Status: u8 {
        const N = 0b10000000;
        const V = 0b01000000;
        const B = 0b00010000;
        const D = 0b00001000;
        const I = 0b00000100;
        const Z = 0b00000010;
        const C = 0b00000001;
    }
}

#[allow(dead_code)]
struct CPU {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: Status,
    mem: [u8; 2048],
}

#[allow(dead_code)]
impl CPU {
    fn new() -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0x0400,
            s: 0xff,
            p: Status::empty(),
            mem: [0; 2048],
        }
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.mem[self.pc as usize];
        self.pc += 1;
        b
    }

    fn read_word(&mut self) -> u16 {
        (self.read_byte() as u16) << 8 | self.read_byte() as u16
    }

    fn push_byte(&mut self, b: u8) {
        self.mem[(0x0100 + self.s as u16) as usize] = b;
        self.s -= 1;
    }

    fn pop_byte(&mut self) -> u8 {
        self.s += 1;
        let b = self.mem[(0x0100 + self.s as u16) as usize];
        b
    }

    fn push_word(&mut self, w: u16) {
        self.push_byte((w >> 8) as u8);
        self.push_byte((w & 0x00ff) as u8);
    }

    fn pop_word(&mut self) -> u16 {
        (self.pop_byte() as u16) | (self.pop_byte() as u16) << 8
    }

    //

    fn update_zn(&mut self, v: u8) {
        self.p.set(Status::Z, v == 0);
        self.p.set(Status::N, v & 0x80 == 0x80);
    }

    // Memory Getters

    fn get_byte(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn get_byte_zpg(&mut self, address: u8) -> u8 {
        self.mem[address as usize]
    }

    fn get_byte_zpgx(&mut self, address: u8) -> u8 {
        self.mem[address.wrapping_add(self.x) as usize]
    }

    fn get_byte_zpgy(&mut self, address: u8) -> u8 {
        self.mem[address.wrapping_add(self.y) as usize]
    }

    fn get_byte_abs(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn get_byte_absx(&mut self, address: u16) -> u8 {
        self.mem[address.wrapping_add(self.x as u16) as usize]
    }

    fn get_byte_absy(&mut self, address: u16) -> u8 {
        self.mem[address.wrapping_add(self.y as u16) as usize]
    }

    fn get_byte_xind(&mut self, address: u8) -> u8 {
        let address = (self.get_byte_zpg(address.wrapping_add(1).wrapping_add(self.x)) as u16) << 8 | self.get_byte_zpg(address.wrapping_add(self.x)) as u16;
        self.mem[address as usize]
    }

    fn get_byte_indy(&mut self, address: u8) -> u8 {
        let address = (self.get_byte_zpg(address.wrapping_add(1)) as u16) << 8 | self.get_byte_zpg(address) as u16;
        self.mem[address.wrapping_add(self.y as u16) as usize]
    }

    // Memory Setters

    fn set_byte(&mut self, address: u16, v: u8) {
        self.mem[address as usize] = v;
    }

    fn set_byte_zpg(&mut self, address: u8, v: u8) {
        self.set_byte(address as u16, v);
    }

    fn set_byte_zpgx(&mut self, address: u8, v: u8) {
        self.set_byte((address.wrapping_add(self.x)) as u16, v);
    }

    fn set_byte_zpgy(&mut self, address: u8, v: u8) {
        self.set_byte((address.wrapping_add(self.y)) as u16, v);
    }

    fn set_byte_abs(&mut self, address: u16, v: u8) {
        self.set_byte(address, v);
    }

    fn set_byte_absx(&mut self, address: u16, v: u8) {
        self.set_byte(address.wrapping_add(self.x as u16), v);
    }

    fn set_byte_absy(&mut self, address: u16, v: u8) {
        self.set_byte(address.wrapping_add(self.y as u16), v);
    }

    fn set_byte_xind(&mut self, address: u8, v: u8) {
        let address = (self.get_byte_zpg(address.wrapping_add(1).wrapping_add(self.x)) as u16) << 8 | self.get_byte_zpg(address.wrapping_add(self.x)) as u16;
        self.set_byte(address, v);
    }

    fn set_byte_indy(&mut self, address: u8, v: u8) {
        let address: u16 = (self.get_byte_zpg(address.wrapping_add(1)) as u16) << 8 | self.get_byte_zpg(address) as u16;
        self.set_byte(address.wrapping_add(self.y as u16), v);
    }

    // Word Shortcuts

    fn get_word(&mut self, address: u16) -> u16 {
        (self.get_byte(address.wrapping_add(1)) as u16) << 8 | self.get_byte(address) as u16
    }
      
    fn set_word(&mut self, address: u16, v: u16) {
        self.set_byte(address+0, v as u8);
        self.set_byte(address+1, (v >> 8) as u8);
    }     

    //

    fn run(&mut self) {
        loop {
            let opcode = self.read_byte();

            match opcode {
                0x00 => {
                    // BRK
                    return;
                }

                0x20 => {
                    // JSR ABS
                    self.push_word(self.pc + 2);
                    self.pc = self.read_word();
                }

                0x60 => {
                    self.pc = self.pop_word();
                }

                0x85 => {
                    // STA ZP
                    let oper = self.read_byte();
                    self.mem[oper as usize] = self.a;
                }

                0x86 => {
                    // STX ZP
                    let oper = self.read_byte();
                    self.mem[oper as usize] = self.x;
                }

                0xA2 => {
                    // LDX IMM
                    self.x = self.read_byte();
                }

                // LDA

                // LDA IMM
                0xA9 => {
                    let operand = self.read_byte();
                    self.a = operand;
                    self.update_zn(self.a);
                }

                // LDA ZP
                0xA5 => {
                    let operand = self.read_byte();
                    self.a = self.get_byte_zpg(operand);
                    self.update_zn(self.a)
                }

                // LDA ZP,X
                0xB5 => {
                    let operand = self.read_byte();
                    self.a = self.get_byte_zpgx(operand);
                    self.update_zn(self.a)
                }

                // LDA ABS
                0xAD => {
                    let operand = self.read_word();
                    self.a = self.get_byte_abs(operand);
                    self.update_zn(self.a)
                }

                // lDA ABS,X
                0xBD => {
                    let operand = self.read_word();
                    self.a = self.get_byte_absx(operand);
                    self.update_zn(self.a)
                }

                // LDA ABS,Y
                0xB9 => {
                    let operand = self.read_word();
                    self.a = self.get_byte_absy(operand);
                    self.update_zn(self.a)
                }

                // LDA (IND,X)
                0xA1 => {
                    let operand = self.read_byte();
                    self.a = self.get_byte_xind(operand);
                    self.update_zn(self.a)
                }

                // LDA (IND),Y
                0xB1 => {
                    let operand = self.read_byte();
                    self.a = self.get_byte_indy(operand);
                    self.update_zn(self.a)
                }
                  
                //

                0xEA => { // NOP
                }

                _ => {
                    panic!("Unknown instruction {}", opcode)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_store() {
        let mut cpu = CPU::new();
        cpu.mem[0x0400] = 0xA9; // LDA #$42
        cpu.mem[0x0401] = 0x42;
        cpu.mem[0x0402] = 0x85; // STA $07
        cpu.mem[0x0403] = 0x07;
        cpu.mem[0x0404] = 0x00; // BRK
        cpu.run();
        assert_eq!(0x0405, cpu.pc);
        assert_eq!(0x42, cpu.a);
        assert_eq!(0x42, cpu.mem[0x0007]);
    }

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
        cpu.run();
        assert_eq!(0x65, cpu.mem[0x0005]);
        assert_eq!(0x02, cpu.mem[0x0006]);
    }

    #[test]
    fn n_is_set() {
        let mut cpu = CPU::new();
        cpu.mem[0x0400] = 0xA9; // LDA #$80
        cpu.mem[0x0401] = 0x80;
        cpu.mem[0x0402] = 0x00;
        cpu.run();
        assert!(cpu.p.contains(Status::N));
        cpu.mem[0x0403] = 0xA9; // LDA #$7F
        cpu.mem[0x0404] = 0x7F;
        cpu.mem[0x0405] = 0x00;
        cpu.run();
        assert!(!cpu.p.contains(Status::N));
    }

    #[test]
    fn z_is_set() {
        let mut cpu = CPU::new();
        cpu.mem[0x0400] = 0xA9; // LDA #$00
        cpu.mem[0x0401] = 0x00;
        cpu.mem[0x0402] = 0x00;
        cpu.run();
        assert!(cpu.p.contains(Status::Z));
        cpu.mem[0x0403] = 0xA9; // LDA #$01
        cpu.mem[0x0404] = 0x01;
        cpu.mem[0x0405] = 0x00;
        cpu.run();
        assert!(!cpu.p.contains(Status::N));
    }

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
}
