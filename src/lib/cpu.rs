use super::gpu;
use super::mbc;
use super::mmu;
use std::collections::HashMap;
use std::fmt;
use std::io::stdin;

const SP_INITIAL_VALUE: u16 = 0xFFFE;

const REG_U8_COUNT: usize = 8;

pub struct CPU {
    gpu: gpu::GPU,
    mmu: mmu::MMU,

    regs: [u8; REG_U8_COUNT],
    reg_sp: u16,
    reg_pc: u16,
    reg_flg: u8,
}

struct Register<'a>(&'a str, usize);

const REG_A: Register = Register("A", 0);
const REG_B: Register = Register("B", 1);
const REG_C: Register = Register("C", 2);
const REG_D: Register = Register("D", 3);
const REG_E: Register = Register("E", 4);
const REG_F: Register = Register("F", 5);
const REG_H: Register = Register("H", 6);
const REG_L: Register = Register("L", 7);

impl CPU {
    pub fn new() -> CPU {
        debug!("Creating new CPU...");
        CPU {
            gpu: gpu::GPU::new(),
            mmu: mmu::MMU::new(),
            regs: [0; REG_U8_COUNT],
            reg_sp: SP_INITIAL_VALUE,
            reg_pc: 0x0100,
            reg_flg: 0x0,
        }
    }

    pub fn exec_inst(&mut self, interactive: bool) -> u8 {
        let mut s = String::new();
        let next_inst = self.nextb();
        let cycles = self.decode_inst(next_inst);
        if interactive {
            stdin().read_line(&mut s);
        }
        cycles
    }

    pub fn read_rom(&mut self, rom: &Vec<u8>) {
        &self.mmu.reset();
        &self.mmu.read_rom(&rom);
    }

    pub fn start(&mut self) {
        loop {
            self.exec_inst(true);
        }
    }

    fn nextb(&mut self) -> u8 {
        let next_byte = self.mmu.rb(self.reg_pc);
        self.inc_pc();
        return next_byte;
    }

    fn nextw(&mut self) -> u16 {
        let lsb = self.nextb();
        let msb = self.nextb();
        return (msb as u16) << 8 | (lsb as u16);
    }

    fn dec_pc(&mut self) -> u16 {
        let mut pc = self.reg_pc;
        self.reg_pc = if pc > 0 { pc - 1 } else { pc };
        self.reg_pc
    }
    fn inc_pc(&mut self) -> u16 {
        let mut pc = self.reg_pc;
        self.reg_pc = if pc < 0xFFFF { pc + 1 } else { 0 };
        self.reg_pc
    }

    fn dec_hl(&mut self) -> u16 {
        let mut hl = self.get_hl();
        return self.set_hl(if hl > 0 { hl - 1 } else { hl });
        
    }
    fn inc_hl(&mut self) -> u16 {
        let mut hl = self.get_hl();
        return self.set_hl(if hl < 0xFFFF { hl + 1 } else { 0 });
    }

    fn get_u16_reg(&self, r1: &Register, r2: &Register) -> u16 {
        let h = self.regs[r1.1];
        let l = self.regs[r2.1];
        ((h as u16) << 8) | (l as u16 & 0xFF)
    }

    fn get_hl(&self) -> u16 {
        self.get_u16_reg(&REG_H, &REG_L)
    }

    fn set_hl(&mut self, value: u16) -> u16 {
        self.regs[REG_H.1] = (value >> 8) as u8;
        self.regs[REG_L.1] = (value & 0x00FF) as u8;
        return value;
    }

    fn get_bc(&self) -> u16 {
        self.get_u16_reg(&REG_B, &REG_C)
    }

    fn set_bc(&mut self, value: u16) -> u16 {
        self.regs[REG_B.1] = (value >> 8) as u8;
        self.regs[REG_C.1] = (value & 0x00FF) as u8;
        return value;
    }

    fn get_de(&self) -> u16 {
        self.get_u16_reg(&REG_D, &REG_E)
    }

    fn set_de(&mut self, value: u16) -> u16 {
        self.regs[REG_D.1] = (value >> 8) as u8;
        self.regs[REG_E.1] = (value & 0x00FF) as u8;
        return value;
    }

    fn get_sp(&self) -> u16 {
        self.reg_sp
    }

    fn set_sp(&mut self, value: u16) -> u16 {
        self.reg_sp = value;
        return value;
    }

    fn ld_reg_byte(&mut self, index: usize, value: u8) {
        let nextb = self.nextb();
        self.set_register(index, nextb);
    }

    fn debug_instr(&self, instr: String) {
        debug!("[CPU][Decoded at PC: 0x{:02x}] {}", self.reg_pc, instr);
    }

    fn set_register(&mut self, index: usize, value: u8) {
        if index >= self.regs.len() {
            panic!("CPU: tried to set register with index out of bounds");
        } else {
            // Actually set the register value
            self.regs[index] = value;
        }
    }

    fn get_register(&self, index: usize) -> u8 {
        if index >= self.regs.len() {
            panic!("CPU: tried to read register with index out of bounds");
        } else {
            // Actually set the register value
            return self.regs[index];
        }
    }

    pub fn decode_inst(&mut self, byte: u8) -> u8 {
        match byte {
            // G0X
            0x01 => {
                // "LD BC,nn"
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (BC) <- 0x{}{}", value_hi, value_lo));
                self.regs[REG_B.1] = value_hi;
                self.regs[REG_C.1] = value_lo;
                3
            }
            0x02 => {
                // "LD (BC),A"
                let value = self.get_register(REG_A.1);
                self.debug_instr(format!("LD (BC) <- A (0x{})", value));
                self.mmu.wb(self.get_bc(), value);
                2
            }
            0x06 => {
                // "LD B,n"
                self.set_register_i(&REG_B);
                2
            }
            0x0A => {
                // "LD A,(BC)"
                let bc = self.get_bc();
                self.debug_instr(format!("LD Reg A <- *(BC) (0x{:02x})", bc));
                self.set_register_from_addr(&REG_A, bc);
                2
            }
            0x0E => {
                // "LD C,n"
                self.set_register_i(&REG_C);
                2
            }
            // G1X
            0x11 => {
                // "LD DE,nn"
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (DE) <- 0x{}{}", value_hi, value_lo));
                self.regs[REG_D.1] = value_hi;
                self.regs[REG_E.1] = value_lo;
                3
            }
            0x12 => {
                // "LD (DE),A"
                let value = self.get_register(REG_A.1);
                self.debug_instr(format!("LD (DE) <- A (0x{})", value));
                self.mmu.wb(self.get_de(), value);
                2
            }
            0x16 => {
                // "LD D,n"
                self.set_register_i(&REG_D);
                2
            }
            0x1A => {
                // "LD A,(DE)"
                let de = self.get_de();
                self.debug_instr(format!("LD Reg A <- *(DE) (0x{:02x})", de));
                self.set_register_from_addr(&REG_A, de);
                2
            }
            0x1E => {
                // "LD E,n"
                self.set_register_i(&REG_E);
                2
            }
            // G2X
            0x21 => {
                // "LD HL,nn"
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (HL) <- 0x{}{}", value_hi, value_lo));
                self.regs[REG_H.1] = value_hi;
                self.regs[REG_L.1] = value_lo;
                3
            }
            0x22 => {
                // "LD (HLI),A / LD (HL+),A / LDI (HL),A"
                let addr = self.get_hl();
                let value = self.regs[REG_A.1];
                self.mmu.wb(addr, value);
                self.debug_instr(format!("LD (HLI=0x{:02x}) <- A (0x{:02x})", addr, value));
                self.inc_hl();
                2
            }
            0x26 => {
                // "LD H,n"
                self.set_register_i(&REG_H);
                2
            }
            0x2A => {
                // LD A,(HLI) / LD A,(HL+), LDI A,(HL)
                let addr = self.get_hl();
                let value = self.mmu.rb(addr);
                self.regs[REG_A.1] = value;
                self.inc_hl();
                self.debug_instr(format!(
                    "LD A <- (HLI)  Addr: 0x{:02x} Value: 0x{:02x}",
                    addr, value
                ));
                2
            }
            0x2E => {
                // "LD L,n"
                self.set_register_i(&REG_L);
                2
            }
            // G3X
            0x31 => {
                // "LD SP,nn"
                let value = self.nextw();
                self.debug_instr(format!("LD (SP) <- 0x{}", value));
                self.reg_sp = value;
                3
            }
            0x32 => {
                // LD (HLD),A / LD (HL-),A / LD (HL),A
                let a = self.regs[REG_A.1];
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg (HLD = {:02x}),A (0x{:02x})", hl, a));
                self.mmu.wb(hl, a);
                self.dec_hl();
                2
            }
            0x36 => {
                // LD (HL), n
                let next_byte = self.nextb();
                self.debug_instr(format!("LD (HL) <- n (0x{})", next_byte));
                self.mmu.wb(self.get_hl(), next_byte);
                3
            }
            0x3A => {
                // "LDD A,(HL) / LD A,(HLD) / LD A,(HL-)"
                self.debug_instr(format!("LDD A,(HL)"));
                let hl = self.get_hl();
                self.debug_instr(format!("LDD A,(HL={:02x})", hl));
                self.set_register_from_addr(&REG_A, hl);
                self.dec_hl();
                2
            }
            0x3E => {
                // "LD A,n"
                self.set_register_i(&REG_A);
                2
            }
            // G4X
            0x40 => {
                // "LD B,B"
                self.set_register_register(&REG_B, &REG_B);
                1
            }
            0x41 => {
                // "LD B,C"
                self.set_register_register(&REG_B, &REG_C);
                1
            }
            0x42 => {
                // "LD B,D"
                self.set_register_register(&REG_B, &REG_D);
                1
            }
            0x43 => {
                // "LD B,E"
                self.set_register_register(&REG_B, &REG_E);
                1
            }
            0x44 => {
                // "LD B,H"
                self.set_register_register(&REG_B, &REG_H);
                1
            }
            0x45 => {
                // "LD B,L"
                self.set_register_register(&REG_B, &REG_L);
                1
            }
            0x46 => {
                // "LD B,(HL)"
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg B <- *(HL) (0x{:02x})", hl));
                self.set_register_from_addr(&REG_B, hl);
                2
            }
            0x47 => {
                // "LD B,A"
                self.set_register_register(&REG_B, &REG_A);
                1
            }
            0x48 => {
                // "LD C,B"
                self.set_register_register(&REG_C, &REG_B);
                1
            }
            0x49 => {
                // "LD C,C"
                self.set_register_register(&REG_C, &REG_C);
                1
            }
            0x4A => {
                // "LD C,D"
                self.set_register_register(&REG_C, &REG_D);
                1
            }
            0x4B => {
                // "LD C,E"
                self.set_register_register(&REG_C, &REG_E);
                1
            }
            0x4C => {
                // "LD C,H"
                self.set_register_register(&REG_C, &REG_H);
                1
            }
            0x4D => {
                // "LD C,L"
                self.set_register_register(&REG_C, &REG_L);
                1
            }
            0x4E => {
                // "LD C,(HL)"
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg C <- *(HL) (0x{:02x})", hl));
                self.set_register_from_addr(&REG_C, hl);
                2
            }
            0x4F => {
                // "LD C,A"
                self.set_register_register(&REG_C, &REG_A);
                1
            }
            // G5X
            0x50 => {
                // "LD D,B"
                self.set_register_register(&REG_D, &REG_B);
                1
            }
            0x51 => {
                // "LD D,C"
                self.set_register_register(&REG_D, &REG_C);
                1
            }
            0x52 => {
                // "LD D,D"
                self.set_register_register(&REG_D, &REG_D);
                1
            }
            0x53 => {
                // "LD D,E"
                self.set_register_register(&REG_D, &REG_E);
                1
            }
            0x54 => {
                // "LD D,H"
                self.set_register_register(&REG_D, &REG_H);
                1
            }
            0x55 => {
                // "LD D,L"
                self.set_register_register(&REG_D, &REG_L);
                1
            }
            0x56 => {
                // "LD D,(HL)"
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg D <- *(HL) (0x{:02x})", hl));
                self.set_register_from_addr(&REG_D, hl);
                2
            }
            0x57 => {
                // "LD D,A"
                self.set_register_register(&REG_D, &REG_A);
                1
            }
            0x58 => {
                // "LD E,B"
                self.set_register_register(&REG_E, &REG_B);
                1
            }
            0x59 => {
                // "LD E,C"
                self.set_register_register(&REG_E, &REG_C);
                1
            }
            0x5A => {
                // "LD E,D"
                self.set_register_register(&REG_E, &REG_D);
                1
            }
            0x5B => {
                // "LD E,E"
                self.set_register_register(&REG_E, &REG_E);
                1
            }
            0x5C => {
                // "LD E,H"
                self.set_register_register(&REG_E, &REG_H);
                1
            }
            0x5D => {
                // "LD E,L"
                self.set_register_register(&REG_E, &REG_L);
                1
            }
            0x5E => {
                // "LD E,(HL)"
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg E <- *(HL) (0x{:02x})", hl));
                self.set_register_from_addr(&REG_E, hl);
                2
            }
            0x5F => {
                // "LD E,A"
                self.set_register_register(&REG_E, &REG_A);
                1
            }
            // G6X
            0x60 => {
                // "LD H,B"
                self.set_register_register(&REG_H, &REG_B);
                1
            }
            0x61 => {
                // "LD H,C"
                self.set_register_register(&REG_H, &REG_C);
                1
            }
            0x62 => {
                // "LD H,D"
                self.set_register_register(&REG_H, &REG_D);
                1
            }
            0x63 => {
                // "LD H,E"
                self.set_register_register(&REG_H, &REG_E);
                1
            }
            0x64 => {
                // "LD H,H"
                self.set_register_register(&REG_H, &REG_H);
                1
            }
            0x65 => {
                // "LD H,L"
                self.set_register_register(&REG_H, &REG_L);
                1
            }
            0x66 => {
                // "LD H,(HL)"
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg H <- *(HL) (0x{:02x})", hl));
                self.set_register_from_addr(&REG_H, hl);
                2
            }
            0x67 => {
                // "LD H,A"
                self.set_register_register(&REG_H, &REG_A);
                1
            }
            0x68 => {
                // "LD L,B"
                self.set_register_register(&REG_L, &REG_B);
                1
            }
            0x69 => {
                // "LD L,C"
                self.set_register_register(&REG_L, &REG_C);
                1
            }
            0x6A => {
                // "LD L,D"
                self.set_register_register(&REG_L, &REG_D);
                1
            }
            0x6B => {
                // "LD L,E"
                self.set_register_register(&REG_L, &REG_E);
                1
            }
            0x6C => {
                // "LD L,H"
                self.set_register_register(&REG_L, &REG_H);
                1
            }
            0x6D => {
                // "LD L,L"
                self.set_register_register(&REG_L, &REG_L);
                1
            }
            0x6E => {
                // "LD L,(HL)"
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg L <- *(HL) (0x{:02x})", hl));
                self.set_register_from_addr(&REG_L, hl);
                2
            }
            0x6F => {
                // "LD L,A"
                self.set_register_register(&REG_L, &REG_A);
                1
            }
            // G7X
            0x70 => {
                // LD (HL), B
                self.debug_instr(format!("LD (HL) <- Reg {}", REG_B.0));
                self.mmu.wb(self.get_hl(), self.get_register(REG_B.1));
                2
            }
            0x71 => {
                // LD (HL), C
                self.debug_instr(format!("LD (HL) <- Reg {}", REG_C.0));
                self.mmu.wb(self.get_hl(), self.get_register(REG_C.1));
                2
            }
            0x72 => {
                // LD (HL), D
                self.debug_instr(format!("LD (HL) <- Reg {}", REG_D.0));
                self.mmu.wb(self.get_hl(), self.get_register(REG_D.1));
                2
            }
            0x73 => {
                // LD (HL), E
                self.debug_instr(format!("LD (HL) <- Reg {}", REG_E.0));
                self.mmu.wb(self.get_hl(), self.get_register(REG_E.1));
                2
            }
            0x74 => {
                // LD (HL), H
                self.debug_instr(format!("LD (HL) <- Reg {}", REG_H.0));
                self.mmu.wb(self.get_hl(), self.get_register(REG_H.1));
                2
            }
            0x75 => {
                // LD (HL), L
                self.debug_instr(format!("LD (HL) <- Reg {}", REG_L.0));
                self.mmu.wb(self.get_hl(), self.get_register(REG_L.1));
                2
            }
            0x77 => {
                // "LD (HL),A"
                let value = self.get_register(REG_A.1);
                self.debug_instr(format!("LD (HL) <- A (0x{})", value));
                self.mmu.wb(self.get_hl(), value);
                2
            }
            0x78 => {
                // "LD A,B"
                self.set_register_register(&REG_A, &REG_B);
                1
            }
            0x79 => {
                // "LD A,C"
                self.set_register_register(&REG_A, &REG_C);
                1
            }
            0x7A => {
                // "LD A,D"
                self.set_register_register(&REG_A, &REG_D);
                1
            }
            0x7B => {
                // "LD A,E"
                self.set_register_register(&REG_A, &REG_E);
                1
            }
            0x7C => {
                // "LD A,H"
                self.set_register_register(&REG_A, &REG_H);
                1
            }
            0x7D => {
                // "LD A,L"
                self.set_register_register(&REG_A, &REG_L);
                1
            }
            0x7E => {
                // "LD A,(HL)"
                let hl = self.get_hl();
                self.debug_instr(format!("LD Reg A <- *(HL) (0x{:02x})", hl));
                self.set_register_from_addr(&REG_A, hl);
                2
            }
            0x7F => {
                // "LD A,A"
                self.set_register_register(&REG_A, &REG_A);
                1
            }
            // G8X
            // G9X
            // GAX
            // GBX
            // GCX
            // GCX
            // GDX
            // GEX
            0xE0 => {
                // "LDH (n),A"
                let addr_offset = self.nextb();
                let value = self.regs[REG_A.1];
                self.mmu.wb(0xFF00 | addr_offset as u16, value);
                3
            }
            0xE2 => {
                // "LD ($FF00+C),A"
                let addr = 0xFF00 | (self.get_register(REG_C.1) as u16);
                let value = self.get_register(REG_A.1);
                self.debug_instr(format!(
                    "LD ($FF00+C) (0x{:02x}) <- A (0x{:02x})",
                    addr, value
                ));
                self.mmu.wb(addr, value);
                2
            }
            0xEA => {
                // "LD (nn),A"
                let addr = self.nextw();
                let value = self.get_register(REG_A.1);
                self.debug_instr(format!("LD (nn) (0x{:02x}) <- A (0x{:02x})", addr, value));
                self.mmu.wb(addr, value);
                4
            }
            // GFX
            0xF0 => {
                // "LDH A,(n)"
                let addr_offset = self.nextb(); 
                let addr = 0xFF00 | addr_offset as u16;
                self.debug_instr(format!("LD A <- 0xFF00 + n (0x{:02x})", addr_offset));
                self.regs[REG_A.1] = self.mmu.rb(addr);
                3
            }
            0xF2 => {
                // "LD A,($FF00 + C)"
                let c = self.get_register(REG_C.1);
                self.set_register_from_addr(&REG_A, 0xFF00 | (c as u16));
                2
            }
            0xF8 => {
                3
            }
            0xF9 => {
                // "LD SP,HL"
                let hl = self.get_hl();
                self.debug_instr(format!("LD SP <- HL (0x{:02x})", hl));
                self.reg_sp = hl;
                2
            }
            0xFA => {
                // "LD A,(nn)"
                let nextw = self.nextw();
                self.set_register_from_addr(&REG_A, nextw);
                4
            }
            _ => {
                warn!(
                    "[Decoded at PC: 0x{:02x}]  OPCODE: 0x{:02x}",
                    self.reg_pc, byte
                );
                return 0;
            }
        }
    }

    fn set_register_i(&mut self, r1: &Register) {
        let next_byte = self.nextb();
        self.debug_instr(format!("LD Reg {} <- 0x{:02x}", r1.0, next_byte));
        self.set_register(r1.1, next_byte)
    }

    fn set_register_register(&mut self, r1: &Register, r2: &Register) {
        let value = self.get_register(r2.1);
        self.debug_instr(format!(
            "LD Reg {} <- *Reg {} (0x{:02x})",
            r1.0, r2.0, value
        ));
        self.set_register(r1.1, value);
    }

    fn set_register_from_addr(&mut self, r1: &Register, addr: u16) {
        self.set_register(r1.1, self.mmu.rb(addr));
    }
}

#[cfg(test)]
#[path = "./cpu_test.rs"]
mod cpu_test;
