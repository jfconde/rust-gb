pub const FLG_ZERO: u8 = 0b10000000;
pub const FLG_SUB: u8 = 0b01000000;
pub const FLG_HCARRY: u8 = 0b00100000;
pub const FLG_CARRY: u8 = 0b00010000;

const SP_INITIAL_VALUE: u16 = 0xFFFE;
const PC_INITIAL_VALUE: u16 = 0x0100;

pub struct CPURegisters {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    flg: CPUFlags,
}

impl CPURegisters {
    pub fn new() -> CPURegisters {
        CPURegisters {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: SP_INITIAL_VALUE,
            pc: PC_INITIAL_VALUE,
            flg: CPUFlags::new(),
        }
    }

    pub fn get_a(&self) -> u8 {
        self.a
    }
    pub fn get_f(&self) -> u8 {
        self.f
    }
    pub fn get_b(&self) -> u8 {
        self.b
    }
    pub fn get_c(&self) -> u8 {
        self.c
    }
    pub fn get_d(&self) -> u8 {
        self.d
    }
    pub fn get_e(&self) -> u8 {
        self.e
    }
    pub fn get_h(&self) -> u8 {
        self.h
    }
    pub fn get_l(&self) -> u8 {
        self.l
    }
    pub fn get_sp(&self) -> u16 {
        self.sp
    }
    pub fn get_pc(&self) -> u16 {
        self.pc
    }
    pub fn get_flg(&mut self) -> &mut CPUFlags {
        &mut self.flg
    }
    pub fn set_a(&mut self, value: u8) {
        self.a = value
    }
    pub fn set_f(&mut self, value: u8) {
        self.f = value
    }
    pub fn set_b(&mut self, value: u8) {
        self.b = value
    }
    pub fn set_c(&mut self, value: u8) {
        self.c = value
    }
    pub fn set_d(&mut self, value: u8) {
        self.d = value
    }
    pub fn set_e(&mut self, value: u8) {
        self.e = value
    }
    pub fn set_h(&mut self, value: u8) {
        self.h = value
    }
    pub fn set_l(&mut self, value: u8) {
        self.l = value
    }
    pub fn set_sp(&mut self, value: u16) {
        self.sp = value
    }
    pub fn set_pc(&mut self, value: u16) {
        self.pc = value
    }
    pub fn inc_pc(&mut self) -> u16 {
        self.pc = self.pc.wrapping_add(1);
        self.pc
    }
    pub fn dec_pc(&mut self) -> u16 {
        self.pc = self.pc.wrapping_sub(1);
        self.pc
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) -> u16 {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
        value
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) -> u16 {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
        value
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) -> u16 {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
        value
    }
}

pub struct CPUFlags {
    value: u8,
}

impl CPUFlags {
    pub fn new() -> CPUFlags {
        CPUFlags { value: 0 }
    }
    pub fn from_value(value: u8) -> CPUFlags {
        CPUFlags {
            value: value >> 4 << 4,
        }
    }

    pub fn set_value(&mut self, value: u8) {
        self.value = value >> 4 << 4;
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }

    pub fn get_flg_zero(&self) -> bool {
        (self.value & FLG_ZERO) > 0
    }

    pub fn set_flg_zero(&mut self, value: bool) -> u8 {
        if value {
            self.value = self.value | FLG_ZERO;
        } else {
            self.value = self.value & (0xFF - FLG_ZERO); // Negate flag mask and & to disable flag.
        }
        self.value
    }

    pub fn get_flg_sub(&self) -> bool {
        (self.value & FLG_SUB) > 0
    }

    pub fn set_flg_sub(&mut self, value: bool) -> u8 {
        if value {
            self.value = self.value | FLG_SUB;
        } else {
            self.value = self.value & (0xFF - FLG_SUB); // Negate flag mask and & to disable flag.
        }
        self.value
    }

    pub fn get_flg_half_carry(&self) -> bool {
        (self.value & FLG_HCARRY) > 0
    }

    pub fn set_flg_half_carry(&mut self, value: bool) -> u8 {
        if value {
            self.value = self.value | FLG_HCARRY;
        } else {
            self.value = self.value & (0xFF - FLG_HCARRY); // Negate flag mask and & to disable flag.
        }
        self.value
    }

    pub fn get_flg_carry(&self) -> bool {
        (self.value & FLG_CARRY) > 0
    }

    pub fn set_flg_carry(&mut self, value: bool) -> u8 {
        if value {
            self.value = self.value | FLG_CARRY;
        } else {
            self.value = self.value & (0xFF - FLG_CARRY); // Negate flag mask and & to disable flag.
        }
        self.value
    }
}
