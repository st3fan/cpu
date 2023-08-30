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

type RegOp = fn(&mut CPU, u8);
type MemOp = fn(&mut CPU, u8) -> u8;
type SetOp = fn(&mut CPU) -> u8;

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

    // Register Operations

    fn adc(&mut self, m: u8) {
        if self.p.contains(Status::D) {
            todo!();
        } else {
            let mut t = self.a as u16 + m as u16;
            if self.p.contains(Status::C) {
                t += 1;
            }
            let r = t as u8;
            self.p.set(Status::C, (t & 0x0100) != 0);
            self.p.set(Status::C, ((self.a^r) & (m^r) & 0x80) != 0);
            self.a = r;
            self.update_zn(self.a);
        }
    }

    fn and(&mut self, m: u8) {
        self.a &= m;
        self.update_zn(self.a);
    }

    fn bit(&mut self, m: u8) {
        let t = self.a & m;
        self.p.set(Status::N, m & 0x80 != 0);
        self.p.set(Status::V, m & 0x80 != 0);
        self.p.set(Status::Z, t == 0);
    }

    fn cmp(&mut self, m: u8) {
        let t = self.a - m;
        self.p.set(Status::C, self.a >= m);
        self.p.set(Status::N, t & 0x80 != 0);
        self.p.set(Status::Z, t == 0);      
    }

    fn cpx(&mut self, m: u8) {
        let t = self.x - m;
        self.p.set(Status::C, self.x >= m);
        self.update_zn(t);
    }

    fn cpy(&mut self, m: u8) {
        let t = self.y - m;
        self.p.set(Status::C, self.y >= m);
        self.update_zn(t);
    }

    fn eor(&mut self, m: u8) {
        self.a ^= m;
        self.update_zn(self.a);
    }

    fn lda(&mut self, m: u8) {
        self.a = m;
        self.update_zn(self.a);
    }

    fn ldx(&mut self, m: u8) {
        self.x = m;
        self.update_zn(self.x);
    }

    fn ldy(&mut self, m: u8) {
        self.y = m;
        self.update_zn(self.y);
    }

    fn ora(&mut self, m: u8) {
        self.a |= m;
        self.update_zn(self.a);
    }

    fn sbc(&mut self, _m: u8) {
        todo!();
    }

    // Memory Operations

    fn asl(&mut self, m: u8) -> u8 {
        self.p.set(Status::B, m & 0x80 != 0);
        let m = m << 1;
        self.p.set(Status::N, m & 0x80 != 0);
        self.p.set(Status::Z, m == 0);
        m
    }

    fn dec(&mut self, m: u8) -> u8 {
        let t = m.wrapping_sub(1);
        self.update_zn(t);
        t
    }

    fn inc(&mut self, m: u8) -> u8 {
        let t = m.wrapping_add(1);
        self.update_zn(t);
        t
    }

    fn lsr(&mut self, m: u8) -> u8 {
        self.p.set(Status::C, m & 0x01 != 0);
        let m = m >> 1;
        self.update_zn(m);
        m
    }

    fn rol(&mut self, m: u8) -> u8 {
        let carry = if self.p.contains(Status::C) { 0x01 } else { 0x00 };
        self.p.set(Status::C, m & 0x80 != 0);
        let m = (m << 1) | carry;
        self.update_zn(m);
        m
    }

    fn ror(&mut self, m: u8) -> u8 {
        let carry = if self.p.contains(Status::C) { 0x01 } else { 0x00 };
        self.p.set(Status::C, m & 0x01 != 0);
        let m = (m << 1) | (carry << 7);
        self.update_zn(m);
        m
    }

    // Memory Update Operations

    fn sta(&mut self) -> u8 {
        self.a
    }

    fn stx(&mut self) -> u8 {
        self.x
    }

    fn sty(&mut self) -> u8 {
        self.y
    }

    fn set_mem_zpg(&mut self, op: SetOp) {
        let a = self.read_byte();
        let r = op(self);
        self.set_byte_zpg(a, r);
    }

    fn set_mem_zpgx(&mut self, op: SetOp) {
        let a = self.read_byte();
        let r = op(self);
        self.set_byte_zpgx(a, r);
    }

    fn set_mem_zpgy(&mut self, op: SetOp) {
        let a = self.read_byte();
        let r = op(self);
        self.set_byte_zpgy(a, r);
    }

    fn set_mem_abs(&mut self, op: SetOp) {
        let a = self.read_word();
        let r = op(self);
        self.set_byte_abs(a, r);
    }

    fn set_mem_absx(&mut self, op: SetOp) {
        let a = self.read_word();
        let r = op(self);
        self.set_byte_absx(a, r);
    }

    fn set_mem_absy(&mut self, op: SetOp) {
        let a = self.read_word();
        let r = op(self);
        self.set_byte_absy(a, r);
    }

    fn set_mem_xind(&mut self, op: SetOp) {
        let a = self.read_byte();
        let r = op(self);
        self.set_byte_xind(a, r);
    }

    fn set_mem_indy(&mut self, op: SetOp) {
        let a = self.read_byte();
        let r = op(self);
        self.set_byte_indy(a, r);
    }

    //

    fn mod_acc(&mut self, op: MemOp) {
        self.a = op(self, self.a);
    }

    fn mod_zpg(&mut self, op: MemOp) {
        let a = self.read_byte();
        let m = self.get_byte_zpg(a);
        let r = op(self, m);
        self.set_byte_zpg(a, r);
    }

    fn mod_zpgx(&mut self, op: MemOp) {
        let a = self.read_byte();
        let m = self.get_byte_zpgx(a);
        let r = op(self, m);
        self.set_byte_zpgx(a, r);
    }

    fn mod_zpgy(&mut self, op: MemOp) {
        let a = self.read_byte();
        let m = self.get_byte_zpgy(a);
        let r = op(self, m);
        self.set_byte_zpgy(a, r);
    }

    fn mod_abs(&mut self, op: MemOp) {
        let a = self.read_word();
        let m = self.get_byte_abs(a);
        let r = op(self, m);
        self.set_byte_abs(a, r);
    }

    fn mod_absx(&mut self, op: MemOp) {
        let a = self.read_word();
        let m = self.get_byte_absx(a);
        let r = op(self, m);
        self.set_byte_absx(a, r);
    }
    
    //

    fn mod_acc_imm(&mut self, op: RegOp) {
        let m = self.read_byte();
        op(self, m);
    }

    fn mod_acc_zpg(&mut self, op: RegOp) {
        let a = self.read_byte();
        let m = self.get_byte_zpg(a);
        op(self, m);
    }

    fn mod_acc_zpgx(&mut self, op: RegOp) {
        let operand = self.read_byte();
        let m = self.get_byte_zpgx(operand);
        op(self, m);
    }

    fn mod_acc_zpgy(&mut self, op: RegOp) {
        let operand = self.read_byte();
        let m = self.get_byte_zpgx(operand);
        op(self, m);
    }

    fn mod_acc_abs(&mut self, op: RegOp) {
        let operand = self.read_word();
        let m = self.get_byte_abs(operand);
        op(self, m);
    }

    fn mod_acc_absx(&mut self, op: RegOp) {
        let operand = self.read_word();
        let m = self.get_byte_absx(operand);
        op(self, m);
    }

    fn mod_acc_absy(&mut self, op: RegOp) {
        let operand = self.read_word();
        let m = self.get_byte_absy(operand);
        op(self, m);
    }

    fn mod_acc_xind(&mut self, op: RegOp) {
        let operand: u8 = self.read_byte();
        let m = self.get_byte_xind(operand);
        op(self, m);
    }

    fn mod_acc_indy(&mut self, op: RegOp) {
        let operand = self.read_byte();
        let m = self.get_byte_indy(operand);
        op(self, m);
    }

    //

    // fn test_mem_imm(&mut self, op: TestOp) {
    //     todo!();
    // }

    // fn test_mem_zpg(&mut self, op: TestOp) {
    //     todo!();
    // }

    // fn test_mem_abs(&mut self, op: TestOp) {
    //     todo!();
    // }

    fn branch(&mut self, flag: Status, set: bool) {
        let offset = (self.read_byte() as i8) as i16;
        if self.p.contains(flag) == set {
            let t = self.pc as i16;
            self.pc = t.wrapping_add(offset) as u16;
        }
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

                // NOP
                0xEA => {
                }

                // Remove above

                0x69 => { self.mod_acc_imm(Self::adc); }
                0x65 => { self.mod_acc_zpg(Self::adc); }
                0x75 => { self.mod_acc_zpgx(Self::adc); }
                0x6D => { self.mod_acc_zpgy(Self::adc); }
                0x7D => { self.mod_acc_absx(Self::adc); }
                0x79 => { self.mod_acc_absy(Self::adc); }
                0x61 => { self.mod_acc_xind(Self::adc); }
                0x71 => { self.mod_acc_indy(Self::adc); }        

                0x29 => { self.mod_acc_imm(Self::and); }
                0x25 => { self.mod_acc_zpg(Self::and); }
                0x35 => { self.mod_acc_zpgx(Self::and); }
                0x2D => { self.mod_acc_zpgy(Self::and); }
                0x3D => { self.mod_acc_absx(Self::and); }
                0x39 => { self.mod_acc_absy(Self::and); }
                0x21 => { self.mod_acc_xind(Self::and); }
                0x31 => { self.mod_acc_indy(Self::and); }        

                0x0A => { self.mod_acc(Self::asl); }
                0x06 => { self.mod_zpg(Self::asl); }
                0x16 => { self.mod_zpgx(Self::asl); }
                0x0E => { self.mod_abs(Self::asl); }
                0x1E => { self.mod_absx(Self::asl); }

                0x24 => { self.mod_acc_zpg(Self::bit); }
                0x2C => { self.mod_acc_abs(Self::bit); }

                /* BCC */ 0x90 => { self.branch(Status::C, false); }
                /* BCS */ 0xB0 => { self.branch(Status::C, true); }
                /* BMI */ 0x30 => { self.branch(Status::N, true); }
                /* BNE */ 0xD0 => { self.branch(Status::Z, false); }
                /* BEQ */ 0xF0 => { self.branch(Status::Z, true); }
                /* BPL */ 0x10 => { self.branch(Status::N, false); }
                /* BVC */ 0x50 => { self.branch(Status::C, false); }
                /* BVS */ 0x70 => { self.branch(Status::C, true); }
                
                // BRK
                
                /* CLC */ 0x18 => { self.p.set(Status::C, false); }
                /* CLD */ 0xD8 => { self.p.set(Status::D, false); }
                /* CLI */ 0x58 => { self.p.set(Status::I, false); }
                /* CLV */ 0xB8 => { self.p.set(Status::V, false); }

                0xC9 => { self.mod_acc_imm(Self::cmp); }
                0xC5 => { self.mod_acc_zpg(Self::cmp); }
                0xD5 => { self.mod_acc_zpgx(Self::cmp); }
                0xCD => { self.mod_acc_abs(Self::cmp); }
                0xDD => { self.mod_acc_absx(Self::cmp); }
                0xD9 => { self.mod_acc_absy(Self::cmp); }
                0xC1 => { self.mod_acc_xind(Self::cmp); }
                0xD1 => { self.mod_acc_indy(Self::cmp); }

                0xE0 => { self.mod_acc_imm(Self::cpx); }
                0xE4 => { self.mod_acc_zpg(Self::cpx); }
                0xEC => { self.mod_acc_abs(Self::cpx); }

                0xC0 => { self.mod_acc_imm(Self::cpy); }
                0xC4 => { self.mod_acc_zpg(Self::cpy); }
                0xCC => { self.mod_acc_abs(Self::cpy); }

                0xD6 => { self.mod_zpgx(Self::dec); }
                0xC6 => { self.mod_zpg(Self::dec); }
                0xCE => { self.mod_abs(Self::dec); }
                0xDE => { self.mod_absx(Self::dec); }

                0xCA => { // DEX
                    self.x = self.x.wrapping_sub(1);
                    self.update_zn(self.x);
                }

                0x88 => { // DEY
                    self.y = self.y.wrapping_sub(1);
                    self.update_zn(self.y);
                }

                0x49 => { self.mod_acc_imm(Self::eor); }
                0x45 => { self.mod_acc_zpg(Self::eor); }
                0x55 => { self.mod_acc_zpgx(Self::eor); }
                0x4D => { self.mod_acc_zpgy(Self::eor); }
                0x5D => { self.mod_acc_absx(Self::eor); }
                0x59 => { self.mod_acc_absy(Self::eor); }
                0x41 => { self.mod_acc_xind(Self::eor); }
                0x51 => { self.mod_acc_indy(Self::eor); }        

                0xE6 => { self.mod_zpg(Self::inc); }
                0xF6 => { self.mod_zpgx(Self::inc); }
                0xEE => { self.mod_abs(Self::inc); }
                0xFE => { self.mod_absx(Self::inc); }

                // INX

                // INY

                // JMP

                // JSR

                0xA9 => { self.mod_acc_imm(Self::lda); }
                0xA5 => { self.mod_acc_zpg(Self::lda); }
                0xB5 => { self.mod_acc_zpgx(Self::lda); }
                0xAD => { self.mod_acc_zpgy(Self::lda); }
                0xBD => { self.mod_acc_absx(Self::lda); }
                0xB9 => { self.mod_acc_absy(Self::lda); }
                0xA1 => { self.mod_acc_xind(Self::lda); }
                0xB1 => { self.mod_acc_indy(Self::lda); }        

                0xA2 => { self.mod_acc_imm(Self::ldx); }
                0xA6 => { self.mod_acc_zpg(Self::ldx); }
                0xB6 => { self.mod_acc_zpgy(Self::ldx); }
                0xAE => { self.mod_acc_abs(Self::ldx); }
                0xBE => { self.mod_acc_absy(Self::ldx); }

                0xA0 => { self.mod_acc_imm(Self::ldy); }
                0xA4 => { self.mod_acc_zpg(Self::ldy); }
                0xB4 => { self.mod_acc_zpgx(Self::ldy); }
                0xAC => { self.mod_acc_abs(Self::ldy); }
                0xBC => { self.mod_acc_absx(Self::ldy); }

                0x4A => { self.mod_acc(Self::lsr); }
                0x46 => { self.mod_zpg(Self::lsr); }
                0x56 => { self.mod_zpgx(Self::lsr); }
                0x4E => { self.mod_abs(Self::lsr); }
                0x5E => { self.mod_absx(Self::lsr); }

                // NOP

                0x09 => { self.mod_acc_imm(Self::ora); }
                0x05 => { self.mod_acc_zpg(Self::ora); }
                0x15 => { self.mod_acc_zpgx(Self::ora); }
                0x0D => { self.mod_acc_zpgy(Self::ora); }
                0x1D => { self.mod_acc_absx(Self::ora); }
                0x19 => { self.mod_acc_absy(Self::ora); }
                0x01 => { self.mod_acc_xind(Self::ora); }
                0x11 => { self.mod_acc_indy(Self::ora); }

                // PHA

                // PHP

                // PLA

                // PLP

                0x2A => { self.mod_acc(Self::rol); }
                0x26 => { self.mod_zpg(Self::rol); }
                0x36 => { self.mod_zpgx(Self::rol); }
                0x2E => { self.mod_abs(Self::rol); }
                0x3E => { self.mod_absx(Self::rol); }

                0x6A => { self.mod_acc(Self::ror); }
                0x66 => { self.mod_zpg(Self::ror); }
                0x76 => { self.mod_zpgx(Self::ror); }
                0x6E => { self.mod_abs(Self::ror); }
                0x7E => { self.mod_absx(Self::ror); }

                // RTI

                // RTS

                0xE9 => { self.mod_acc_imm(Self::sbc); }
                0xE5 => { self.mod_acc_zpg(Self::sbc); }
                0xF5 => { self.mod_acc_zpgx(Self::sbc); }
                0xED => { self.mod_acc_zpgy(Self::sbc); }
                0xFD => { self.mod_acc_absx(Self::sbc); }
                0xF9 => { self.mod_acc_absy(Self::sbc); }
                0xE1 => { self.mod_acc_xind(Self::sbc); }
                0xF1 => { self.mod_acc_indy(Self::sbc); }

                /* SEC */ 0x38 => { self.p.set(Status::C, true); }
                /* SED */ 0xF8 => { self.p.set(Status::D, true); }
                /* SEI */ 0x78 => { self.p.set(Status::I, true); }

                0x85 => { self.set_mem_zpg(Self::sta); }
                0x95 => { self.set_mem_zpgx(Self::sta); }
                0x8D => { self.set_mem_abs(Self::sta); }
                0x9D => { self.set_mem_absx(Self::sta); }
                0x99 => { self.set_mem_absy(Self::sta); }
                0x81 => { self.set_mem_xind(Self::sta); }
                0x91 => { self.set_mem_indy(Self::sta); }

                0x96 => { self.set_mem_zpg(Self::stx); }
                0x86 => { self.set_mem_zpgy(Self::stx); }
                0x8E => { self.set_mem_abs(Self::stx); }

                0x84 => { self.set_mem_zpg(Self::sty); }
                0x94 => { self.set_mem_zpgx(Self::sty); }
                0x8C => { self.set_mem_abs(Self::sty); }

                // TAX

                // TAY

                // TSX

                // TXA

                // TXS

                // TYA

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
