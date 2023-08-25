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
        CPU { a: 0, x: 0 ,y: 0, pc: 0x0400, s: 0xff, p: 0x00, mem: [0; 2048] }
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.mem[self.pc as usize];
        self.pc += 1;
        b
    }

    fn run(&mut self) {
        loop {
            let opcode = self.read_byte();

            match opcode {
                0xA9 => { // LDA IMM
                    let oper = self.read_byte();
                    self.a = oper;
                }

                0x85 => { // STA ZP
                    let oper = self.read_byte();
                    self.mem[oper as usize] = self.a;
                }

                0x00 => { // BRK
                    return
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
}
