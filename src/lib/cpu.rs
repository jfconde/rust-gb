use super::cpu_registers;
use super::gpu;
use super::mmu;
use crate::lib::cpu_registers::{FLG_CARRY, FLG_ZERO};
use std::io::stdin;

const SP_INITIAL_VALUE: u16 = 0xFFFE;
const REG_U8_COUNT: usize = 8;

pub struct CPU {
    gpu: gpu::GPU,
    mmu: mmu::MMU,
    registers: cpu_registers::CPURegisters,
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
            registers: cpu_registers::CPURegisters::new(),
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
        debug!(
            "[Registers] SP: {:02x}, PC: {:02x}, ",
            self.registers.get_sp(),
            self.registers.get_pc(),
        );
    }

    pub fn read_rom(&mut self, rom: &Vec<u8>) {
        let _ = &self.mmu.reset();
        let _ = &self.mmu.read_rom(&rom);
    }

    pub fn start(&mut self) {
        let mut input = String::new();
        loop {
            if true
            /* TODO: interactive check */
            {
                if input.eq("d\n") {
                    self.dump_status();
                } else {
                    self.exec_inst();
                }
                input.clear();
                match stdin().read_line(&mut input) {
                    Ok(read_count) => {}
                    Err(error) => panic!("Error reading next line."),
                }
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
        self.registers
            .set_hl(self.registers.get_hl().wrapping_sub(1))
    }
    fn inc_hl(&mut self) -> u16 {
        self.registers
            .set_hl(self.registers.get_hl().wrapping_add(1))
    }

    fn debug_instr(&self, instr: String) {
        debug!(
            "[CPU][Decoded at PC: 0x{:02x}] {}",
            self.registers.get_pc() - 2,
            instr
        );
    }

    fn alu16_add_with_carry(
        &mut self,
        a: u16,
        n: u16,
        carry: u16,
        half_carry: u16,
        set_zero: bool,
    ) -> u16 {
        let result = a.wrapping_add(n);
        let flags = self.registers.get_flags();
        if set_zero && result == 0 {
            flags.set_zero(result == 0);
        }
        flags.set_sub(false);
        flags.set_hcarry(((a & half_carry) as u32 + (n & half_carry) as u32) > half_carry as u32);
        flags.set_carry(((a & carry) as u32 + (n & carry) as u32) > carry as u32);

        result
    }

    fn alu16_add(&mut self, a: u16, n: u16, set_zero: bool) -> u16 {
        self.alu16_add_with_carry(a, n, 0xFFFF, 0xFFF, set_zero)
    }

    fn alu8_add_with_carry(
        &mut self,
        a: u8,
        n: u8,
        set_zero: bool,
        carry: u8,
        half_carry: u8,
    ) -> u8 {
        let result = a.wrapping_add(n);
        // Set flags
        if set_zero && result == 0 {
            self.registers.get_flags().set_zero(result == 0);
        }
        self.registers.get_flags().set_sub(false);
        self.registers
            .get_flags()
            .set_hcarry(((a & half_carry) as u16 + (n & half_carry) as u16) > half_carry as u16);

        self.registers
            .get_flags()
            .set_carry(((a & carry) as u16 + (n & carry) as u16) > carry as u16);

        result
    }

    fn alu8_add(&mut self, a: u8, n: u8, set_zero: bool) -> u8 {
        self.alu8_add_with_carry(a, n, set_zero, 0xFF, 0xF)
    }

    fn alu8_adc_with_carry(
        &mut self,
        a: u8,
        n: u8,
        carry_enabled: bool,
        set_zero: bool,
        carry_mask: u8,
        half_carry_mask: u8,
    ) -> u8 {
        let c: u8 = if carry_enabled { 1 } else { 0 };
        let result = a.wrapping_add(n).wrapping_add(c);
        // Set flags
        if set_zero && result == 0 {
            self.registers.get_flags().set_zero(result == 0);
        }
        self.registers.get_flags().set_sub(false);
        self.registers.get_flags().set_hcarry(
            ((a & half_carry_mask) as u16 + (n & half_carry_mask) as u16 + c as u16)
                > half_carry_mask as u16,
        );

        self.registers.get_flags().set_carry(
            ((a & carry_mask) as u16 + (n & carry_mask) as u16 + c as u16) > carry_mask as u16,
        );

        result
    }

    fn alu8_adc(&mut self, a: u8, n: u8, carry_enabled: bool, set_zero: bool) -> u8 {
        self.alu8_adc_with_carry(a, n, carry_enabled, set_zero, 0xFF, 0xF)
    }

    fn alu8_r(&mut self, value: u8, right: bool) -> u8 {
        let mut rotated_bit: u8 = 0;
        let mut rotated: u8 = 0;
        let mut mask: u8 = 0;
        if right {
            rotated_bit = value & 0b1;
            rotated = value.rotate_right(1);
            mask = (rotated_bit as u8) * FLG_CARRY | ((rotated == 0) as u8) * FLG_ZERO;
        } else {
            rotated_bit = (value & 0b1000_0000) >> 7;
            rotated = value.rotate_left(1);
            mask = rotated_bit * FLG_CARRY | (((rotated == 0) as u8) * FLG_ZERO);
        }

        self.registers.get_flags().set_value(mask);
        rotated
    }

    fn alu8_rc(&mut self, value: u8, right: bool) -> u8 {
        let mut rotated_bit: u8 = 0;
        let mut rotated: u8 = 0;
        let mut mask: u8 = 0;
        let carry = self.registers.get_flags().get_carry() as u8;

        if right {
            rotated_bit = value & 0b1;
            rotated = (value >> 1) | (carry * 0b1000_0000);
            mask = (rotated_bit as u8) * FLG_CARRY | ((rotated == 0) as u8) * FLG_ZERO;
        } else {
            rotated_bit = (value & 0b1000_0000) >> 7;
            rotated = ((value & 0b0111_1111) << 1) | carry;
            mask = rotated_bit * FLG_CARRY | (((rotated == 0) as u8) * FLG_ZERO);
        }

        self.registers.get_flags().set_value(mask);
        rotated
    }

    pub fn decode_inst(&mut self, byte: u8) -> u8 {
        match byte {
            // G0X
            0x00 /* NOP */ => {
                self.debug_instr(String::from("NOP"));
                1
            }
            0x01 /* LD BC,nn */ => {
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (BC) <- nn 0x{:02x}{:02x}", value_hi, value_lo));
                self.registers.set_b(value_hi);
                self.registers.set_c(value_lo);
                3
            }
            0x02 /* LD BC,A */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (BC) <- A (0x{:02x})", value));
                self.mmu.wb(self.registers.get_bc(), value);
                2
            }
            0x03 /* INC BC */ => {
                let value = self.registers.get_bc();
                self.debug_instr(format!("INC BC (0x{:02x})", value));
                self.registers.set_bc(value.wrapping_add(1));
                2
            }
            0x04 /* INC B */ => {
                let value = self.registers.get_b();
                self.debug_instr(format!("INC B (0x{:02x})", value));
                let inc = value.wrapping_add(1);
                self.registers.set_b(inc);
                let flags = self.registers.get_flags();
                flags.set_sub(false);
                flags.set_hcarry(((value & 0xF) + 1) > 0xF);
                flags.set_zero(inc == 0);
                1
            }
            0x05 /* DEC B */ => {
                let value = self.registers.get_b();
                let dec = value.wrapping_sub(1);
                self.debug_instr(format!("DEC B (0x{:02x})", value));
                self.registers.set_b(dec);
                let flags = self.registers.get_flags();
                flags.set_sub(true);
                flags.set_hcarry((value & 0xF) == 0);
                flags.set_zero(dec == 0);
                1
            }
            0x06 /* LD B,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("LD (B) <- (0x{:02x})", value));
                self.registers.set_b(value);
                2
            }
            0x07 /* RLCA */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("RLCA (0x{:02x})", value));
                let rotated = self.alu8_r(value, false);
                self.registers.set_a(rotated);
                1
            }
            0x08 /* LD (nn),SP */ => {
                let value = self.nextw();
                let sp = self.registers.get_sp();
                self.debug_instr(format!("LD (nn=0x{:04x}),SP", value));
                self.mmu.wb(value, sp as u8);
                self.mmu.wb(value.wrapping_add(1), (sp >> 8) as u8);
                5
            }
            0x09 /* ADD HL,BC */ => {
                let value = self.registers.get_bc();
                let hl = self.registers.get_hl();
                let sum = self.alu16_add(hl, value, false);
                self.registers.set_hl(sum);
                2
            }
            0x0a /* LD A,(BC) */ => {
                let bc = self.registers.get_bc();
                self.debug_instr(format!("LD Reg A <- *(BC) (0x{:02x})", bc));
                self.registers.set_a(self.mmu.rb(bc));
                2
            }
            0x0b /* DEC BC */ => {
                let value = self.registers.get_bc();
                self.debug_instr(format!("DEC BC (0x{:02x})", value));
                self.registers.set_bc(value.wrapping_sub(1));
                2
            }
            0x0c /* INC C */ => {
                let value = self.registers.get_c();
                self.debug_instr(format!("INC C (0x{:02x})", value));
                let inc = value.wrapping_add(1);
                self.registers.set_c(inc);
                self.registers.get_flags().set_sub(false);
                self.registers
                    .get_flags()
                    .set_hcarry(((value & 0xF) + 1) > 0xF);
                self.registers.get_flags().set_zero(inc == 0);
                1
            }
            0x0d /* DEC C */ => {
                let value = self.registers.get_c();
                let dec = value.wrapping_sub(1);
                self.debug_instr(format!("DEC C (0x{:02x})", value));
                self.registers.set_c(dec);
                self.registers.get_flags().set_sub(true);
                self.registers.get_flags().set_hcarry((value & 0xF) == 0);
                self.registers.get_flags().set_zero(dec == 0);
                1
            }
            0x0e /* LD C,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("LD (C) <- (0x{:02x})", value));
                self.registers.set_c(value);
                2
            }
            0x0f /* RLCA */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("RRCA (0x{:02x})", value));
                let rotated = self.alu8_r(value, true);
                self.registers.set_a(rotated);
                1
            }
            // G1X
            0x11 /* LD DE,nn */ => {
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (DE) <- 0x{:02x}{:02x}", value_hi, value_lo));
                self.registers.set_d(value_hi);
                self.registers.set_e(value_lo);
                3
            }
            0x12 /* LD (DE),A */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (DE) <- A (0x{:02x})", value));
                self.mmu.wb(self.registers.get_de(), value);
                2
            }
            0x13 /* INC DE */ => {
                let value = self.registers.get_de();
                self.debug_instr(format!("INC DE (0x{:02x})", value));
                self.registers.set_de(value.wrapping_add(1));
                2
            }
            0x14 /* INC D */ => {
                let value = self.registers.get_d();
                self.debug_instr(format!("INC D (0x{:02x})", value));
                let inc = value.wrapping_add(1);
                self.registers.set_d(inc);
                self.registers.get_flags().set_sub(false);
                self.registers
                    .get_flags()
                    .set_hcarry(((value & 0xF) + 1) > 0xF);
                self.registers.get_flags().set_zero(inc == 0);
                1
            }
            0x15 /* DEC D */ => {
                let value = self.registers.get_d();
                let dec = value.wrapping_sub(1);
                self.debug_instr(format!("DEC D (0x{:02x})", value));
                self.registers.set_d(dec);
                self.registers.get_flags().set_sub(true);
                self.registers.get_flags().set_hcarry((value & 0xF) == 0);
                self.registers.get_flags().set_zero(dec == 0);
                1
            }
            0x16 /* LD D,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("LD (D) <- (0x{:02x})", value));
                self.registers.set_d(value);
                2
            }
            0x17 /* RLA */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("RLA (0x{:02x})", value));
                let rotated = self.alu8_rc(value, false);
                self.registers.set_a(rotated);
                1
            }
            0x19 /* ADD HL,DE */ => {
                let value = self.registers.get_de();
                let hl = self.registers.get_hl();
                let sum = self.alu16_add(hl, value, false);
                self.registers.set_hl(sum);
                2
            }
            0x1a /* LD A,(DE) */ => {
                let de = self.registers.get_de();
                self.debug_instr(format!("LD Reg A <- *(DE) (0x{:02x})", de));
                let value = self.mmu.rb(de);
                self.registers.set_a(value);
                2
            }
            0x1b /* DEC DE */ => {
                let value = self.registers.get_de();
                self.debug_instr(format!("DEC DE (0x{:02x})", value));
                self.registers.set_de(value.wrapping_sub(1));
                2
            }
            0x1c /* INC E */ => {
                let value = self.registers.get_e();
                self.debug_instr(format!("INC E (0x{:02x})", value));
                let inc = value.wrapping_add(1);
                self.registers.set_e(inc);
                self.registers.get_flags().set_sub(false);
                self.registers
                    .get_flags()
                    .set_hcarry(((value & 0xF) + 1) > 0xF);
                self.registers.get_flags().set_zero(inc == 0);
                1
            }
            0x1d /* DEC D */ => {
                let value = self.registers.get_e();
                let dec = value.wrapping_sub(1);
                self.debug_instr(format!("DEC E (0x{:02x})", value));
                self.registers.set_e(dec);
                self.registers.get_flags().set_sub(true);
                self.registers.get_flags().set_hcarry((value & 0xF) == 0);
                self.registers.get_flags().set_zero(dec == 0);
                1
            }
            0x1e /* LD E,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("LD (E) <- (0x{:02x})", value));
                self.registers.set_e(value);
                2
            }
            0x1f /* RLA */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("RRA (0x{:02x})", value));
                let rotated = self.alu8_rc(value, true);
                self.registers.set_a(rotated);
                1
            }
            // G2X
            0x21 /* LD HL,nn */ => {
                let value_lo = self.nextb();
                let value_hi = self.nextb();
                self.debug_instr(format!("LD (HL) <- 0x{:02x}{:02x}", value_hi, value_lo));
                self.registers.set_h(value_hi);
                self.registers.set_l(value_lo);
                3
            }
            0x22 /* LD (HLI),A / LD (HL+),A / LDI (HL),A */ => {
                let addr = self.registers.get_hl();
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (HLI=0x{:02x}) <- A (0x{:02x})", addr, value));
                self.mmu.wb(addr, value);
                self.inc_hl();
                2
            }
            0x23 /* INC HL */ => {
                let value = self.registers.get_hl();
                self.debug_instr(format!("INC HL (0x{:02x})", value));
                self.registers.set_hl(value.wrapping_add(1));
                2
            }
            0x24 /* INC H */ => {
                let value = self.registers.get_h();
                self.debug_instr(format!("INC H (0x{:02x})", value));
                let inc = value.wrapping_add(1);
                self.registers.set_h(inc);
                self.registers.get_flags().set_sub(false);
                self.registers
                    .get_flags()
                    .set_hcarry(((value & 0xF) + 1) > 0xF);
                self.registers.get_flags().set_zero(inc == 0);
                1
            }
            0x25 /* DEC H */ => {
                let value = self.registers.get_h();
                let dec = value.wrapping_sub(1);
                self.debug_instr(format!("DEC H (0x{:02x})", value));
                self.registers.set_h(dec);
                self.registers.get_flags().set_sub(true);
                self.registers.get_flags().set_hcarry((value & 0xF) == 0);
                self.registers.get_flags().set_zero(dec == 0);
                1
            }
            0x26 /* LD H,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("LD (H) <- (0x{:02x})", value));
                self.registers.set_h(value);
                2
            }
            0x29 /* ADD HL,HL */ => {
                let hl = self.registers.get_hl();
                let sum = self.alu16_add(hl, hl, false);
                self.registers.set_hl(sum);
                2
            }
            0x2a /* LD A,(HLI) / LD A,(HL+), LDI A,(HL) */ => {
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
            0x2b /* DEC HL */ => {
                let value = self.registers.get_hl();
                self.debug_instr(format!("DEC HL (0x{:02x})", value));
                self.registers.set_hl(value.wrapping_sub(1));
                2
            }
            0x2c /* INC L */ => {
                let value = self.registers.get_l();
                self.debug_instr(format!("INC H (0x{:02x})", value));
                let inc = value.wrapping_add(1);
                self.registers.set_l(inc);
                self.registers.get_flags().set_sub(false);
                self.registers
                    .get_flags()
                    .set_hcarry(((value & 0xF) + 1) > 0xF);
                self.registers.get_flags().set_zero(inc == 0);
                1
            }
            0x2d /* DEC L */ => {
                let value = self.registers.get_l();
                let dec = value.wrapping_sub(1);
                self.debug_instr(format!("DEC L (0x{:02x})", value));
                self.registers.set_l(dec);
                self.registers.get_flags().set_sub(true);
                self.registers.get_flags().set_hcarry((value & 0xF) == 0);
                self.registers.get_flags().set_zero(dec == 0);
                1
            }
            0x2e /* LD L,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("LD (L) <- (0x{:02x})", value));
                self.registers.set_l(value);
                2
            }
            // G3X
            0x31 /* LD SP,nn */ => {
                let value = self.nextw();
                self.debug_instr(format!("LD (SP) <- 0x{:02x}", value));
                self.registers.set_sp(value);
                3
            }
            0x32 /* LD (HLD),A / LD (HL-),A / LD (HL),A */ => {
                let a = self.registers.get_a();
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg (HLD = {:02x}),A (0x{:02x})", hl, a));
                self.mmu.wb(hl, a);
                self.dec_hl();
                2
            }
            0x33 /* INC SP */ => {
                let value = self.registers.get_sp();
                self.debug_instr(format!("INC SP (0x{:02x})", value));
                self.registers.set_sp(value.wrapping_add(1));
                2
            }
            0x34 /* INC (HL) */ => {
                let hl = self.registers.get_hl();
                let value = self.mmu.rb(hl);
                self.debug_instr(format!("INC (HL) (0x{:02x}) at address {:02x}", value, hl));
                let inc = value.wrapping_add(1);
                self.mmu.wb(hl, inc);
                self.registers.get_flags().set_sub(false);
                self.registers
                    .get_flags()
                    .set_hcarry(((value & 0xF) + 1) > 0xF);
                self.registers.get_flags().set_zero(inc == 0);
                1
            }
            0x35 /* DEC (HL) */ => {
                let hl = self.registers.get_hl();
                let value = self.mmu.rb(hl);
                self.debug_instr(format!("DEC (HL) (0x{:02x}) at address {:02x}", value, hl));
                let dec = value.wrapping_sub(1);
                self.mmu.wb(hl, dec);
                self.registers.get_flags().set_sub(true);
                self.registers.get_flags().set_hcarry((value & 0xF) == 0);
                self.registers.get_flags().set_zero(dec == 0);
                1
            }
            0x36 /* LD (HL), n */ => {
                let next_byte = self.nextb();
                self.debug_instr(format!("LD (HL) <- n (0x{:02x})", next_byte));
                self.mmu.wb(self.registers.get_hl(), next_byte);
                3
            }
            0x39 /* ADD HL,SP */ => {
                let value = self.registers.get_sp();
                let hl = self.registers.get_hl();
                let sum = self.alu16_add(hl, value, false);
                self.registers.set_hl(sum);
                2
            }
            0x3a /* LDD A,(HL) / LD A,(HLD) / LD A,(HL-) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LDD A,(HL={:02x})", hl));
                self.registers.set_a(self.mmu.rb(hl));
                self.dec_hl();
                2
            }
            0x3b /* DEC SP */ => {
                let value = self.registers.get_sp();
                self.debug_instr(format!("DEC SP (0x{:02x})", value));
                self.registers.set_sp(value.wrapping_sub(1));
                2
            }
            0x3c /* INC A */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("INC A (0x{:02x})", value));
                let inc = value.wrapping_add(1);
                self.registers.set_a(inc);
                self.registers.get_flags().set_sub(false);
                self.registers
                    .get_flags()
                    .set_hcarry(((value & 0xF) + 1) > 0xF);
                self.registers.get_flags().set_zero(inc == 0);
                1
            }
            0x3d /* DEC A */ => {
                let value = self.registers.get_a();
                let dec = value.wrapping_sub(1);
                self.debug_instr(format!("DEC A (0x{:02x})", value));
                self.registers.set_a(dec);
                self.registers.get_flags().set_sub(true);
                self.registers.get_flags().set_hcarry((value & 0xF) == 0);
                self.registers.get_flags().set_zero(dec == 0);
                1
            }
            0x3e /* LD A,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("LD (A) <- (0x{:02x})", value));
                self.registers.set_a(value);
                2
            }
            // G4X
            0x40 /* LD B,B */ => {
                self.debug_instr(format!("LD (B) <- (B={:02x})", self.registers.get_b()));
                1
            }
            0x41 /* LD B,C */ => {
                self.debug_instr(format!("LD (B) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_b(self.registers.get_c());
                1
            }
            0x42 /* LD B,D */ => {
                self.debug_instr(format!("LD (B) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_b(self.registers.get_d());
                1
            }
            0x43 /* LD B,E */ => {
                self.debug_instr(format!("LD (B) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_b(self.registers.get_e());
                1
            }
            0x44 /* LD B,H */ => {
                self.debug_instr(format!("LD (B) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_b(self.registers.get_h());
                1
            }
            0x45 /* LD B,L */ => {
                self.debug_instr(format!("LD (B) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_b(self.registers.get_l());
                1
            }
            0x46 /* LD B,(HL) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg B <- *(HL) (0x{:02x})", hl));
                self.registers.set_b(self.mmu.rb(hl));
                2
            }
            0x47 /* LD B,A */ => {
                self.debug_instr(format!("LD (B) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_b(self.registers.get_a());
                1
            }
            0x48 /* LD C,B */ => {
                self.debug_instr(format!("LD (C) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_c(self.registers.get_b());
                1
            }
            0x49 /* LD C,C */ => {
                self.debug_instr(format!("LD (C) <- (C={:02x})", self.registers.get_c()));
                1
            }
            0x4a /* LD C,D */ => {
                self.debug_instr(format!("LD (C) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_c(self.registers.get_d());
                1
            }
            0x4b /* LD C,E */ => {
                self.debug_instr(format!("LD (C) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_c(self.registers.get_e());
                1
            }
            0x4c /* LD C,H */ => {
                self.debug_instr(format!("LD (C) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_c(self.registers.get_h());
                1
            }
            0x4d /* LD C,L */ => {
                self.debug_instr(format!("LD (C) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_c(self.registers.get_l());
                1
            }
            0x4e /* LD C,(HL) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg C <- *(HL) (0x{:02x})", hl));
                self.registers.set_c(self.mmu.rb(hl));
                2
            }
            0x4f /* LD C,A */ => {
                self.debug_instr(format!("LD (C) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_c(self.registers.get_a());
                1
            }
            // G5X
            0x50 /* LD D,B */ => {
                self.debug_instr(format!("LD (D) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_d(self.registers.get_b());
                1
            }
            0x51 /* LD D,C */ => {
                self.debug_instr(format!("LD (D) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_d(self.registers.get_c());
                1
            }
            0x52 /* LD D,D */ => {
                self.debug_instr(format!("LD (D) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_d(self.registers.get_d());
                1
            }
            0x53 /* LD D,E */ => {
                self.debug_instr(format!("LD (D) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_d(self.registers.get_e());
                1
            }
            0x54 /* LD D,H */ => {
                self.debug_instr(format!("LD (D) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_d(self.registers.get_h());
                1
            }
            0x55 /* LD D,L */ => {
                self.debug_instr(format!("LD (D) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_d(self.registers.get_l());
                1
            }
            0x56 /* LD D,(HL) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg D <- *(HL) (0x{:02x})", hl));
                self.registers.set_d(self.mmu.rb(hl));
                2
            }
            0x57 /* LD D,A */ => {
                self.debug_instr(format!("LD (D) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_d(self.registers.get_a());
                1
            }
            0x58 /* LD E,B */ => {
                self.debug_instr(format!("LD (E) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_e(self.registers.get_b());
                1
            }
            0x59 /* LD E,C */ => {
                self.debug_instr(format!("LD (E) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_e(self.registers.get_c());
                1
            }
            0x5a /* LD E,D */ => {
                self.debug_instr(format!("LD (E) <- (D={:02x})", self.registers.get_b()));
                self.registers.set_e(self.registers.get_d());
                1
            }
            0x5b /* LD E,E */ => {
                self.debug_instr(format!("LD (E) <- (E={:02x})", self.registers.get_e()));
                1
            }
            0x5c /* LD E,H */ => {
                self.debug_instr(format!("LD (E) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_e(self.registers.get_h());
                1
            }
            0x5d /* LD E,L */ => {
                self.debug_instr(format!("LD (E) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_e(self.registers.get_l());
                1
            }
            0x5e /* LD E,(HL) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg E <- *(HL) (0x{:02x})", hl));
                self.registers.set_e(self.mmu.rb(hl));
                2
            }
            0x5f /* LD E,A */ => {
                self.debug_instr(format!("LD (E) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_e(self.registers.get_a());
                1
            }
            // G6X
            0x60 /* LD H,B */ => {
                self.debug_instr(format!("LD (H) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_h(self.registers.get_b());
                1
            }
            0x61 /* LD H,C */ => {
                self.debug_instr(format!("LD (H) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_h(self.registers.get_c());
                1
            }
            0x62 /* LD H,D */ => {
                self.debug_instr(format!("LD (H) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_h(self.registers.get_d());
                1
            }
            0x63 /* LD H,E */ => {
                self.debug_instr(format!("LD (H) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_h(self.registers.get_e());
                1
            }
            0x64 /* LD H,H */ => {
                self.debug_instr(format!("LD (H) <- (H={:02x})", self.registers.get_h()));
                1
            }
            0x65 /* LD H,L */ => {
                self.debug_instr(format!("LD (H) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_h(self.registers.get_l());
                1
            }
            0x66 /* LD H,(HL) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg H <- *(HL) (0x{:02x})", hl));
                self.registers.set_h(self.mmu.rb(hl));
                2
            }
            0x67 /* LD H,A */ => {
                self.debug_instr(format!("LD (H) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_h(self.registers.get_a());
                1
            }
            0x68 /* LD L,B */ => {
                self.debug_instr(format!("LD (L) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_l(self.registers.get_b());
                1
            }
            0x69 /* LD L,C */ => {
                self.debug_instr(format!("LD (L) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_l(self.registers.get_c());
                1
            }
            0x6a /* LD L,D */ => {
                self.debug_instr(format!("LD (L) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_l(self.registers.get_d());
                1
            }
            0x6b /* LD L,E */ => {
                self.debug_instr(format!("LD (L) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_l(self.registers.get_e());
                1
            }
            0x6c /* LD L,H */ => {
                self.debug_instr(format!("LD (L) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_l(self.registers.get_h());
                1
            }
            0x6d /* LD L,L */ => {
                self.debug_instr(format!("LD (L) <- (L={:02x})", self.registers.get_l()));
                1
            }
            0x6e /* LD L,(HL) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD Reg L <- *(HL) (0x{:02x})", hl));
                self.registers.set_l(self.mmu.rb(hl));
                2
            }
            0x6f /* LD L,A */ => {
                self.debug_instr(format!("LD (L) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_l(self.registers.get_a());
                1
            }
            // G7X
            0x70 /* LD (HL), B */ => {
                self.debug_instr(format!(
                    "LD (HL={:02x}) <- (B={:02x})",
                    self.registers.get_hl(),
                    self.registers.get_b()
                ));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_b());
                2
            }
            0x71 /* LD (HL), C */ => {
                self.debug_instr(format!(
                    "LD (HL={:02x}) <- (C={:02x})",
                    self.registers.get_hl(),
                    self.registers.get_c()
                ));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_c());
                2
            }
            0x72 /* LD (HL), D */ => {
                self.debug_instr(format!(
                    "LD (HL={:02x}) <- (D={:02x})",
                    self.registers.get_hl(),
                    self.registers.get_d()
                ));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_d());
                2
            }
            0x73 /* LD (HL), E */ => {
                self.debug_instr(format!(
                    "LD (HL={:02x}) <- (E={:02x})",
                    self.registers.get_hl(),
                    self.registers.get_e()
                ));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_e());
                2
            }
            0x74 /* LD (HL), H */ => {
                self.debug_instr(format!(
                    "LD (HL={:02x}) <- (H={:02x})",
                    self.registers.get_hl(),
                    self.registers.get_h()
                ));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_h());
                2
            }
            0x75 /* LD (HL), L */ => {
                self.debug_instr(format!(
                    "LD (HL={:02x}) <- (L={:02x})",
                    self.registers.get_hl(),
                    self.registers.get_l()
                ));
                self.mmu.wb(self.registers.get_hl(), self.registers.get_l());
                2
            }
            0x77 /* LD (HL),A */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (HL) <- (A=0x{:02x})", value));
                self.mmu.wb(self.registers.get_hl(), value);
                2
            }
            0x78 /* LD A,B */ => {
                self.debug_instr(format!("LD (A) <- (B={:02x})", self.registers.get_b()));
                self.registers.set_a(self.registers.get_b());
                1
            }
            0x79 /* LD A,C */ => {
                self.debug_instr(format!("LD (A) <- (C={:02x})", self.registers.get_c()));
                self.registers.set_a(self.registers.get_c());
                1
            }
            0x7a /* LD A,D */ => {
                self.debug_instr(format!("LD (A) <- (D={:02x})", self.registers.get_d()));
                self.registers.set_a(self.registers.get_d());
                1
            }
            0x7b /* LD A,E */ => {
                self.debug_instr(format!("LD (A) <- (E={:02x})", self.registers.get_e()));
                self.registers.set_a(self.registers.get_e());
                1
            }
            0x7c /* LD A,H */ => {
                self.debug_instr(format!("LD (A) <- (H={:02x})", self.registers.get_h()));
                self.registers.set_a(self.registers.get_h());
                1
            }
            0x7d /* LD A,L */ => {
                self.debug_instr(format!("LD (A) <- (L={:02x})", self.registers.get_l()));
                self.registers.set_a(self.registers.get_l());
                1
            }
            0x7e /* LD A,(HL) */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD (A) <- *(HL) (0x{:02x})", hl));
                self.registers.set_a(self.mmu.rb(hl));
                2
            }
            0x7f /* LD A,A */ => {
                self.debug_instr(format!("LD (A) <- (A={:02x})", self.registers.get_a()));
                self.registers.set_a(self.registers.get_a());
                1
            }
            // G8X
            0x80 /* ADD A,B */ => {
                self.debug_instr(format!("ADD (A, B={:02x})", self.registers.get_b()));
                let result = self.alu8_add(self.registers.get_a(), self.registers.get_b(), true);
                self.registers.set_a(result);
                1
            }
            0x81 /* ADD A,C */ => {
                self.debug_instr(format!("ADD (A, C={:02x})", self.registers.get_c()));
                let result = self.alu8_add(self.registers.get_a(), self.registers.get_c(), true);
                self.registers.set_a(result);
                1
            }
            0x82 /* ADD A,D */ => {
                self.debug_instr(format!("ADD (A, D={:02x})", self.registers.get_d()));
                let result = self.alu8_add(self.registers.get_a(), self.registers.get_d(), true);
                self.registers.set_a(result);
                1
            }
            0x83 /* ADD A,E */ => {
                self.debug_instr(format!("ADD (A, E={:02x})", self.registers.get_e()));
                let result = self.alu8_add(self.registers.get_a(), self.registers.get_e(), true);
                self.registers.set_a(result);
                1
            }
            0x84 /* ADD A,H */ => {
                self.debug_instr(format!("ADD (A, H={:02x})", self.registers.get_h()));
                let result = self.alu8_add(self.registers.get_a(), self.registers.get_h(), true);
                self.registers.set_a(result);
                1
            }
            0x85 /* ADD A,L */ => {
                self.debug_instr(format!("ADD (A, L={:02x})", self.registers.get_l()));
                let result = self.alu8_add(self.registers.get_a(), self.registers.get_l(), true);
                self.registers.set_a(result);
                1
            }
            0x86 /* ADD A,(HL) */ => {
                let hl = self.registers.get_hl();
                let content = self.mmu.rb(hl);
                self.debug_instr(format!(
                    "ADD (A, *(HL) (0x{:04x}, content={:02x})",
                    hl, content
                ));
                let result = self.alu8_add(self.registers.get_a(), content, true);
                self.registers.set_a(result);
                2
            }
            0x87 /* ADD A,A */ => {
                let value = self.registers.get_a();
                self.debug_instr(format!("ADD (A, A={:02x})", value));
                let result = self.alu8_add(value, value, true);
                self.registers.set_a(result);
                1
            }
            0x88 /* ADC A,B */ => {
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!(
                    "ADC (A, B={:02x}) - Carry flag: {}",
                    self.registers.get_b(),
                    carry
                ));

                let result =
                    self.alu8_adc(self.registers.get_a(), self.registers.get_b(), carry, true);

                self.registers.set_a(result);
                1
            }
            0x89 /* ADC A,C */ => {
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!(
                    "ADC (A, C={:02x}) - Carry flag: {}",
                    self.registers.get_c(),
                    carry
                ));

                let result =
                    self.alu8_adc(self.registers.get_a(), self.registers.get_c(), carry, true);

                self.registers.set_a(result);
                1
            }
            0x8a /* ADC A,D */ => {
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!(
                    "ADC (A, D={:02x}) - Carry flag: {}",
                    self.registers.get_d(),
                    carry
                ));

                let result =
                    self.alu8_adc(self.registers.get_a(), self.registers.get_d(), carry, true);

                self.registers.set_a(result);
                1
            }
            0x8b /* ADC A,E */ => {
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!(
                    "ADC (A, E={:02x}) - Carry flag: {}",
                    self.registers.get_e(),
                    carry
                ));

                let result =
                    self.alu8_adc(self.registers.get_a(), self.registers.get_e(), carry, true);

                self.registers.set_a(result);
                1
            }
            0x8c /* ADC A,H */ => {
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!(
                    "ADC (A, H={:02x}) - Carry flag: {}",
                    self.registers.get_h(),
                    carry
                ));

                let result =
                    self.alu8_adc(self.registers.get_a(), self.registers.get_h(), carry, true);

                self.registers.set_a(result);
                1
            }
            0x8d /* ADC A,L */ => {
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!(
                    "ADC (A, L={:02x}) - Carry flag: {}",
                    self.registers.get_l(),
                    carry
                ));

                let result =
                    self.alu8_adc(self.registers.get_a(), self.registers.get_l(), carry, true);

                self.registers.set_a(result);
                1
            }
            0x8e /* ADC A,(HL) */ => {
                let carry = self.registers.get_flags().get_carry();
                let hl = self.registers.get_hl();
                let content = self.mmu.rb(hl);
                self.debug_instr(format!(
                    "ADC (A, *(HL) (0x{:04x}, content={:02x})- Carry flag: {}",
                    hl, content, carry
                ));
                let result = self.alu8_adc(self.registers.get_a(), content, carry, true);
                self.registers.set_a(result);
                2
            }
            0x8f /* ADC A,A */ => {
                let value = self.registers.get_a();
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!("ADC (A, A={:02x}) - Carry flag: {}", value, carry));
                let result = self.alu8_adc(value, value, carry, true);
                self.registers.set_a(result);
                1
            }
            // G9X
            // GAX
            // GBX
            // GCX
            0xc6 /* ADD A,n */ => {
                let value = self.nextb();
                self.debug_instr(format!("ADD (A, n={:02x})", value));
                let result = self.alu8_add(self.registers.get_a(), value, true);
                self.registers.set_a(result);
                2
            }
            0xce /* ADC A,n */ => {
                let value = self.nextb();
                let carry = self.registers.get_flags().get_carry();
                self.debug_instr(format!("ADD (A, n={:02x}) - Carry flag {}", value, carry));
                let result = self.alu8_adc(self.registers.get_a(), value, carry, true);
                self.registers.set_a(result);
                2
            }
            // GCX
            // GDX
            // GEX
            0xe0 /* LDH (n),A */ => {
                let addr_offset = self.nextb();
                let value = self.registers.get_a();
                self.debug_instr(format!(
                    "LDH ( $FF00 + {:02x}) <- (A={:02x})",
                    addr_offset, value
                ));
                self.mmu.wb(0xFF00 | addr_offset as u16, value);
                3
            }
            0xe2 /* LD ($FF00+C),A */ => {
                let addr = 0xFF00 | (self.registers.get_c() as u16);
                let value = self.registers.get_a();
                self.debug_instr(format!(
                    "LD ($FF00+C) (0x{:02x}) <- A (0x{:02x})",
                    addr, value
                ));
                self.mmu.wb(addr, value);
                2
            }
            0xea /* LD (nn),A */ => {
                let addr = self.nextw();
                let value = self.registers.get_a();
                self.debug_instr(format!("LD (nn) (0x{:02x}) <- A (0x{:02x})", addr, value));
                self.mmu.wb(addr, value);
                4
            }
            // GFX
            0xf0 /* LDH A,(n) */ => {
                let addr_offset = self.nextb();
                let addr = 0xFF00 | addr_offset as u16;
                self.debug_instr(format!("LD A <- 0xFF00 + n (0x{:02x})", addr_offset));
                self.registers.set_a(self.mmu.rb(addr));
                3
            }
            0xf2 /* LD A,($FF00 + C) */ => {
                let c = self.registers.get_c();
                self.debug_instr(format!("LD A <- 0xFF00 + (C={:02x})", c));
                let value = self.mmu.rb(0xFF00 | (c as u16));
                self.registers.set_a(value);
                2
            }
            0xf8 /* LD HL,SP+(n) */ => {
                let n = self.nextb() as u8 as i8;
                let sp = self.registers.get_sp();
                self.debug_instr(format!("LDHL HL <- SP (0x{:04x}) + n (0x{:02x})", sp, n));
                self.registers.get_flags().set_zero(false); // The previous alu call will not reset zero
                let result = self.alu16_add_with_carry(sp, n as u16, 0xFF, 0xF, false);
                self.registers.set_hl(result);
                3
            }
            0xf9 /* LD SP,HL */ => {
                let hl = self.registers.get_hl();
                self.debug_instr(format!("LD SP <- HL (0x{:02x})", hl));
                self.registers.set_sp(hl);
                2
            }
            0xfa /* LD A,(nn) */ => {
                let nextw = self.nextw();
                let value = self.mmu.rb(nextw);
                self.debug_instr(format!("LD A <- (*0x{:02x}=0x{:02x}", nextw, value));
                self.registers.set_a(value);
                4
            }
            _ => {
                warn!(
                    "[Decoded at PC: 0x{:02x}]  OPCODE: 0x{:02x}",
                    self.registers.get_pc(),
                    byte
                );
                return 0;
            }
        }
    }
}

#[cfg(test)]
#[path = "./cpu_test.rs"]
mod cpu_test;
