// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/

//
// This is a 6502 emulator with the following memory layout:
//
//  0x0000 - 0x00ff RAM Zero Page
//  0x0100 - 0x01ff RAM Stack
//  0x0200 - 0x03ff RAM General Use
//  0x0400 - 0x07ff ROM
//
// Only the following instructions have been implemented:
//
//  0x00             BRK
//  0xA2  0xXX       LDX IMM
//  0xA9  0xXX       LDA IMM
//  0x85  0xXX       STA ZP
//  0x86  0xXX       STX ZP
//  0x20  0xLL 0xHH  JSR ABS
//  0x60             RTS
//  0xEA             NOP
//  0xE6             INC ZP
//

#[allow(dead_code)]
struct CPU {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: u8,
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
            p: 0x00,
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

    fn run(&mut self) {
        loop {
            let opcode = self.read_byte();

            println!("Executing {:x}", opcode);

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

                0xA9 => {
                    // LDA IMM
                    self.a = self.read_byte();
                }

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
}
