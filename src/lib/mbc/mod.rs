pub mod mbc0;
use mbc0::MBC0;

pub const ADDR_CARTRIDGE_TYPE: u16 = 0x0147;
pub const C_TYPE_MBC0: u8 = 0x00;
/*pub const C_TYPE_MBC1: u8 = 0x01;
pub const C_TYPE_MBC1_RAM: u8 = 0x02;
pub const C_TYPE_MBC1_RAM_BATT: u8 = 0x03;
pub const C_TYPE_MBC2: u8 = 0x05;
pub const C_TYPE_MBC2_BATT: u8 = 0x06;*/

pub trait MBC {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, byte: u8);
    fn get_type(&self) -> &str;
}

pub enum MbcType {
    MBC0(MBC0)
}

impl MBC for MbcType {
    fn read(&self, addr: u16) -> u8 {
        match *self {
            MbcType::MBC0(ref mbc0) => mbc0.read(addr)
        }
    }

    fn write(&mut self, addr: u16, byte: u8) {
        match *self {
            MbcType::MBC0(ref mut mbc0) => mbc0.write(addr, byte)
        }
    }

    fn get_type(&self) -> &str {
        match * self {
            MbcType::MBC0(ref mbc0) => mbc0.get_type()
        }
    }
}


pub struct MBCBuilder {}

impl MBCBuilder {
    pub fn from_rom (rom: &Vec<u8>) -> Option<MbcType> {
        let cart_type_byte = rom[ADDR_CARTRIDGE_TYPE as usize];
        return match cart_type_byte {
            C_TYPE_MBC0 => Some(MbcType::MBC0(MBC0::from_rom(rom))),
            _ => None
        }
    }
}
