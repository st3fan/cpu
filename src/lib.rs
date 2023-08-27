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

    // TODO Needs test
    fn read_byte(&mut self) -> u8 {
        let b = self.mem[self.pc as usize];
        self.pc += 1;
        b
    }

    // TODO Needs test
    fn read_word(&mut self) -> u16 {
         self.read_byte() as u16 | (self.read_byte() as u16) << 8
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
mod cpu_tests;

#[cfg(test)]
mod ins_tests;

#[cfg(test)]
mod mem_tests;
