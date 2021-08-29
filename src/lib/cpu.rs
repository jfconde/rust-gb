use super::gpu;
use super::mbc;
use super::mmu;
use super::cpu_registers;
use std::collections::HashMap;
use std::fmt;
use std::io::stdin;

const SP_INITIAL_VALUE: u16 = 0xFFFE;

const REG_U8_COUNT: usize = 8;

pub struct CPU {
    gpu: gpu::GPU,
    mmu: mmu::MMU,
    registers: cpu_registers::CPURegisters
}


fn merge_bytes(upper_byte: u8, lower_byte: u8) -> u16 {
    (upper_byte as u16) << 8 | (lower_byte as u16)
}

impl CPU {
    pub fn new() -> CPU {
        debug!("Creating new CPU...");
        CPU {
            gpu: gpu::GPU::new(),
            mmu: mmu::MMU::new(),
            registers: cpu_registers::CPURegisters::new()
        }
    }


    pub fn exec_inst(&mut self) -> u8 {
        let next_inst = self.nextb();
        let cycles = self.decode_inst(next_inst);
        cycles
    }

    pub fn dump_status(&mut self) {
        debug!("[CPU Status]:");
        debug!("[Registers] A: {:02x}, B: {:02x}, C: {:02x}, D: {:02x}, E: {:02x}, F: {:02x}, H: {:02x}, L: {:02x}, ",
            self.registers.get_a(),
            self.registers.get_b(),
            self.registers.get_c(),
            self.registers.get_d(),
            self.registers.get_e(),
            self.registers.get_f(),
            self.registers.get_h(),
            self.registers.get_l(),
        );
        debug!("[Registers] SP: {:02x}, PC: {:02x}, ",
            self.registers.get_sp(),
            self.registers.get_pc(),
        );
    }

    pub fn read_rom(&mut self, rom: &Vec<u8>) {
        &self.mmu.reset();
        &self.mmu.read_rom(&rom);
    }

    pub fn start(&mut self) {
        let mut input = String::new();
        loop {
            if true /* TODO: interactive check */ {
                if input.eq("d\n") {
                    self.dump_status();
                }else{
                    self.exec_inst();
                }
                input.clear();
                stdin().read_line(&mut input);
            }
        }
    }

    fn nextb(&mut self) -> u8 {
        let next_byte = self.mmu.rb(self.registers.get_pc());
        self.registers.inc_pc();
        return next_byte;
    }

    fn nextw(&mut self) -> u16 {
        let lsb = self.nextb();
        let msb = self.nextb();
        return merge_bytes(msb, lsb);
    }

    fn dec_hl(&mut self) -> u16 {
        self.registers.set_hl(self.registers.get_hl().wrapping_sub(1))
    }
    fn inc_hl(&mut self) -> u16 {
        self.registers.set_hl(self.registers.get_hl().wrapping_add(1))
    }

    fn debug_instr(&self, instr: String) {
        debug!("[CPU][Decoded at PC: 0x{:02x}] {}", self.registers.get_pc(), instr);
    }

    fn alu8_add(&mut self, a: u8, n: u8) {
        /*let result = a.wrapping_add(n);
        self.set_flag_z(result == 0);*/
    }

    pub fn decode_inst(&mut self, byte: u8) -> u8 {
        match byte {
            // G0X
            0x01 => {
                // "LD BC,nn"
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (BC) <- nn 0x{:02x}{:02x}", value_hi, value_lo));
                self.registers.set_b(value_hi);
                self.registers.set_c(value_lo);
                3
            }
            0x02 => {
                // "LD BC,A"
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (BC) <- A (0x{:02x})", value));
                self.mmu.wb(self.registers.get_bc(), value);
                2
            }
            0x06 => {
                // "LD B,n"
                let value = self.nextb();
                self.debug_instr(format!("LD (B) <- (0x{:02x})", value));
                self.registers.set_b(value);
                2
            }
            0x0A => {
                // "LD A,(BC)"
                let bc = self.registers.get_bc();
                self.debug_instr(format!("LD Reg A <- *(BC) (0x{:02x})", bc));
                self.registers.set_a(self.mmu.rb(bc));
                2
            }
            0x0E => {
                // "LD C,n"
                let value = self.nextb();
                self.debug_instr(format!("LD (C) <- (0x{:02x})", value));
                self.registers.set_c(value);
                2
            }
            // G1X
            0x11 => {
                // "LD DE,nn"
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (DE) <- 0x{:02x}{:02x}", value_hi, value_lo));
                self.registers.set_d(value_hi);
                self.registers.set_e(value_lo);
                3
            }
            0x12 => {
                // "LD (DE),A"
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (DE) <- A (0x{:02x})", value));
                self.mmu.wb(self.registers.get_de(), value);
                2
            }
            0x16 => {
                // "LD D,n"
                let value = self.nextb();
                self.debug_instr(format!("LD (D) <- (0x{:02x})", value));
                self.registers.set_d(value);
                2
            }
            0x1A => {
                // "LD A,(DE)"
                let de = self.registers.get_de();
                self.debug_instr(format!("LD Reg A <- *(DE) (0x{:02x})", de));
                let value = self.mmu.rb(de);
                self.registers.set_a(value);
                2
            }
            0x1E => {
                // "LD E,n"
                let value = self.nextb();
                self.debug_instr(format!("LD (E) <- (0x{:02x})", value));
                self.registers.set_e(value);
                2
            }
            // G2X
            0x21 => {
                // "LD HL,nn"
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (HL) <- 0x{:02x}{:02x}", value_hi, value_lo));
                self.registers.set_h(value_hi);
                self.registers.set_l(value_lo);
                3
            }
            0x22 => {
                // "LD (HLI),A / LD (HL+),A / LDI (HL),A"
                let addr = self.registers.get_hl();
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (HLI=0x{:02x}) <- A (0x{:02x})", addr, value));
                self.mmu.wb(addr, value);
                self.inc_hl();
                2
            }
            0x26 => {
                // "LD H,n"
                let value = self.nextb();
                self.debug_instr(format!("LD (H) <- (0x{:02x})", value));
                self.registers.set_h(value);
                2
            }
            0x2A => {
                // LD A,(HLI) / LD A,(HL+), LDI A,(HL)
                let addr = self.registers.get_hl();
                let value = self.mmu.rb(addr);
                self.debug_instr(format!(
                    "LD A <- (HLI)  Addr: 0x{:02x} Value: 0x{:02x}",
                    addr, value
                ));
                self.registers.set_a(value);
                self.inc_hl();
                2
            }
            0x2E => {
                // "LD L,n"
                let value = self.nextb();
                self.debug_instr(format!("LD (L) <- (0x{:02x})", value));
                self.registers.set_l(value);
                2
            }
            // G3X
            0x31 => {
                // "LD SP,nn"
                let value = self.nextw();
                self.debug_instr(format!("LD (SP) <- 0x{:02x}", value));
                self.registers.set_sp(value);
                3
            }
            0x32 => {
                // LD (HLD),A / LD (HL-),A / LD (HL),A
                let a = self.registers.get_a();
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg (HLD = {:02x}),A (0x{:02x})", hl, a));
                self.mmu.wb(hl, a);
                self.dec_hl();
                2
            }
            0x36 => {
                // LD (HL), n
                let next_byte = self.nextb();
                self.debug_instr(format!("LD (HL) <- n (0x{:02x})", next_byte));
                self.mmu.wb(self.registers.get_hl(), next_byte);
                3
            }
            0x3A => {
                // "LDD A,(HL) / LD A,(HLD) / LD A,(HL-)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LDD A,(HL={:02x})", hl));
                self.registers.set_a(self.mmu.rb(hl));
                self.dec_hl();
                2
            }
            0x3E => {
                // "LD A,n"
                let value = self.nextb();
                self.debug_instr(format!("LD (A) <- (0x{:02x})", value));
                self.registers.set_a(value);
                2
            }
            // G4X
            0x40 => {
                // "LD B,B"
                self.debug_instr(format!("LD (B) <- (B={:02x})", self.registers.get_b()));
                1
            }
            0x41 => {
                // "LD B,C"
                self.debug_instr(format!("LD (B) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_b(self.registers.get_c());
                1
            }
            0x42 => {
                // "LD B,D"
                self.debug_instr(format!("LD (B) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_b(self.registers.get_d());
                1
            }
            0x43 => {
                // "LD B,E"
                self.debug_instr(format!("LD (B) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_b(self.registers.get_e());
                1
            }
            0x44 => {
                // "LD B,H"
                self.debug_instr(format!("LD (B) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_b(self.registers.get_h());
                1
            }
            0x45 => {
                // "LD B,L"
                self.debug_instr(format!("LD (B) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_b(self.registers.get_l());
                1
            }
            0x46 => {
                // "LD B,(HL)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg B <- *(HL) (0x{:02x})", hl));
                self.registers.set_b(self.mmu.rb(hl));
                2
            }
            0x47 => {
                // "LD B,A"
                self.debug_instr(format!("LD (B) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_b(self.registers.get_a());
                1
            }
            0x48 => {
                // "LD C,B"
                self.debug_instr(format!("LD (C) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_c(self.registers.get_b());
                1
            }
            0x49 => {
                // "LD C,C"
                self.debug_instr(format!("LD (C) <- (C={:02x})", self.registers.get_c()));
                1
            }
            0x4A => {
                // "LD C,D"
                self.debug_instr(format!("LD (C) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_c(self.registers.get_d());
                1
            }
            0x4B => {
                // "LD C,E"
                self.debug_instr(format!("LD (C) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_c(self.registers.get_e());
                1
            }
            0x4C => {
                // "LD C,H"
                self.debug_instr(format!("LD (C) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_c(self.registers.get_h());
                1
            }
            0x4D => {
                // "LD C,L"
                self.debug_instr(format!("LD (C) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_c(self.registers.get_l());
                1
            }
            0x4E => {
                // "LD C,(HL)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg C <- *(HL) (0x{:02x})", hl));
                self.registers.set_c(self.mmu.rb(hl));
                2
            }
            0x4F => {
                // "LD C,A"
                self.debug_instr(format!("LD (C) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_c(self.registers.get_a());
                1
            }
            // G5X
            0x50 => {
                // "LD D,B"
                self.debug_instr(format!("LD (D) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_d(self.registers.get_b());
                1
            }
            0x51 => {
                // "LD D,C"
                self.debug_instr(format!("LD (D) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_d(self.registers.get_c());
                1
            }
            0x52 => {
                // "LD D,D"
                self.debug_instr(format!("LD (D) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_d(self.registers.get_d());
                1
            }
            0x53 => {
                // "LD D,E"
                self.debug_instr(format!("LD (D) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_d(self.registers.get_e());
                1
            }
            0x54 => {
                // "LD D,H"
                self.debug_instr(format!("LD (D) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_d(self.registers.get_h());
                1
            }
            0x55 => {
                // "LD D,L"
                self.debug_instr(format!("LD (D) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_d(self.registers.get_l());
                1
            }
            0x56 => {
                // "LD D,(HL)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg D <- *(HL) (0x{:02x})", hl));
                self.registers.set_d(self.mmu.rb(hl));
                2
            }
            0x57 => {
                // "LD D,A"
                self.debug_instr(format!("LD (D) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_d(self.registers.get_a());
                1
            }
            0x58 => {
                // "LD E,B"
                self.debug_instr(format!("LD (E) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_e(self.registers.get_b());
                1
            }
            0x59 => {
                // "LD E,C"
                self.debug_instr(format!("LD (E) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_e(self.registers.get_c());
                1
            }
            0x5A => {
                // "LD E,D"
                self.debug_instr(format!("LD (E) <- (D={:02x})", self.registers.get_b()));
                self.registers.set_e(self.registers.get_d());
                1
            }
            0x5B => {
                // "LD E,E"
                self.debug_instr(format!("LD (E) <- (E={:02x})", self.registers.get_e()));
                1
            }
            0x5C => {
                // "LD E,H"
                self.debug_instr(format!("LD (E) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_e(self.registers.get_h());
                1
            }
            0x5D => {
                // "LD E,L"
                self.debug_instr(format!("LD (E) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_e(self.registers.get_l());
                1
            }
            0x5E => {
                // "LD E,(HL)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg E <- *(HL) (0x{:02x})", hl));
                self.registers.set_e(self.mmu.rb(hl));
                2
            }
            0x5F => {
                // "LD E,A"
                self.debug_instr(format!("LD (E) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_e(self.registers.get_a());
                1
            }
            // G6X
            0x60 => {
                // "LD H,B"
                self.debug_instr(format!("LD (H) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_h(self.registers.get_b());
                1
            }
            0x61 => {
                // "LD H,C"
                self.debug_instr(format!("LD (H) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_h(self.registers.get_c());
                1
            }
            0x62 => {
                // "LD H,D"
                self.debug_instr(format!("LD (H) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_h(self.registers.get_d());
                1
            }
            0x63 => {
                // "LD H,E"
                self.debug_instr(format!("LD (H) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_h(self.registers.get_e());
                1
            }
            0x64 => {
                // "LD H,H"
                self.debug_instr(format!("LD (H) <- (H={:02x})", self.registers.get_h()));
                1
            }
            0x65 => {
                // "LD H,L"
                self.debug_instr(format!("LD (H) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_h(self.registers.get_l());
                1
            }
            0x66 => {
                // "LD H,(HL)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg H <- *(HL) (0x{:02x})", hl));
                self.registers.set_h(self.mmu.rb(hl));
                2
            }
            0x67 => {
                // "LD H,A"
                self.debug_instr(format!("LD (H) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_h(self.registers.get_a());
                1
            }
            0x68 => {
                // "LD L,B"
                self.debug_instr(format!("LD (L) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_l(self.registers.get_b());
                1
            }
            0x69 => {
                // "LD L,C"
                self.debug_instr(format!("LD (L) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_l(self.registers.get_c());
                1
            }
            0x6A => {
                // "LD L,D"
                self.debug_instr(format!("LD (L) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_l(self.registers.get_d());
                1
            }
            0x6B => {
                // "LD L,E"
                self.debug_instr(format!("LD (L) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_l(self.registers.get_e());
                1
            }
            0x6C => {
                // "LD L,H"
                self.debug_instr(format!("LD (L) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_l(self.registers.get_h());
                1
            }
            0x6D => {
                // "LD L,L"
                self.debug_instr(format!("LD (L) <- (L={:02x})", self.registers.get_l()));
                1
            }
            0x6E => {
                // "LD L,(HL)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg L <- *(HL) (0x{:02x})", hl));
                self.registers.set_l(self.mmu.rb(hl));
                2
            }
            0x6F => {
                // "LD L,A"
                self.debug_instr(format!("LD (L) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_l(self.registers.get_a());
                1
            }
            // G7X
            0x70 => {
                // LD (HL), B
                self.debug_instr(format!("LD (HL={:02x}) <- (B={:02x})", self.registers.get_hl(), self.registers.get_b()));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_b());
                2
            }
            0x71 => {
                // LD (HL), C
                self.debug_instr(format!("LD (HL={:02x}) <- (C={:02x})", self.registers.get_hl(), self.registers.get_c()));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_c());
                2
            }
            0x72 => {
                // LD (HL), D
                self.debug_instr(format!("LD (HL={:02x}) <- (D={:02x})", self.registers.get_hl(), self.registers.get_d()));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_d());
                2
            }
            0x73 => {
                // LD (HL), E
                self.debug_instr(format!("LD (HL={:02x}) <- (E={:02x})", self.registers.get_hl(), self.registers.get_e()));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_e());
                2
            }
            0x74 => {
                // LD (HL), H
                self.debug_instr(format!("LD (HL={:02x}) <- (H={:02x})", self.registers.get_hl(), self.registers.get_h()));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_h());
                2
            }
            0x75 => {
                // LD (HL), L
                self.debug_instr(format!("LD (HL={:02x}) <- (L={:02x})", self.registers.get_hl(), self.registers.get_l()));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_l());
                2
            }
            0x77 => {
                // "LD (HL),A"
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (HL) <- (A=0x{:02x})", value));
                self.mmu.wb(self.registers.get_hl(), value);
                2
            }
            0x78 => {
                // "LD A,B"
                self.debug_instr(format!("LD (A) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_a(self.registers.get_b());
                1
            }
            0x79 => {
                // "LD A,C"
                self.debug_instr(format!("LD (A) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_a(self.registers.get_c());
                1
            }
            0x7A => {
                // "LD A,D"
                self.debug_instr(format!("LD (A) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_a(self.registers.get_d());
                1
            }
            0x7B => {
                // "LD A,E"
                self.debug_instr(format!("LD (A) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_a(self.registers.get_e());
                1
            }
            0x7C => {
                // "LD A,H"
                self.debug_instr(format!("LD (A) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_a(self.registers.get_h());
                1
            }
            0x7D => {
                // "LD A,L"
                self.debug_instr(format!("LD (A) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_a(self.registers.get_l());
                1
            }
            0x7E => {
                // "LD A,(HL)"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg A <- *(HL) (0x{:02x})", hl));
                self.registers.set_a(self.mmu.rb(hl));
                2
            }
            0x7F => {
                // "LD A,A"
                self.debug_instr(format!("LD (A) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_a(self.registers.get_a());
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
                let value = self.registers.get_a();
                self.debug_instr(format!("LDH ( $FF00 + {:02x}) <- (A={:02x})", addr_offset, value));
                self.mmu.wb(0xFF00 | addr_offset as u16, value);
                3
            }
            0xE2 => {
                // "LD ($FF00+C),A"
                let addr = 0xFF00 | (self.registers.get_c() as u16);
                let value = self.registers.get_a();
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
                let value = self.registers.get_a();
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
                self.registers.set_a(self.mmu.rb(addr));
                3
            }
            0xF2 => {
                // "LD A,($FF00 + C)"
                let c = self.registers.get_c();
                self.debug_instr(format!("LD A <- 0xFF00 + (C={:02x})", c));
                let value = self.mmu.rb(0xFF00 | (c as u16));
                self.registers.set_a(value);
                2
            }
            0xF8 => {
                // "LD HL,SP+(n)"
                let addr_offset = self.nextb(); 
                let addr = self.registers.get_sp() + addr_offset as u16;
                self.debug_instr(format!("LDHL HL <- SP (0x{:04x}) + n (0x{:02x})", self.registers.get_sp(), addr));
                // TODO: finish                
                3
            }
            0xF9 => {
                // "LD SP,HL"
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD SP <- HL (0x{:02x})", hl));
                self.registers.set_sp(hl);
                2
            }
            0xFA => {
                // "LD A,(nn)"
                let nextw = self.nextw();
                let value = self.mmu.rb(nextw);
                self.debug_instr(format!("LD A <- (*0x{:02x}=0x{:02x}", nextw, value));
                self.registers.set_a(value);
                4
            }
            _ => {
                warn!(
                    "[Decoded at PC: 0x{:02x}]  OPCODE: 0x{:02x}",
                    self.registers.get_pc(), byte
                );
                return 0;
            }
        }
    }
}

#[cfg(test)]
#[path = "./cpu_test.rs"]
mod cpu_test;
 