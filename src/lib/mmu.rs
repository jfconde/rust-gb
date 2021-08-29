//use std::convert::TryFrom;
use super::ram;
use super::mbc::MBCBuilder;
use super::mbc::mbc;
use super::mbc::mbcType;

const MEMORY_SIZE: usize = 65535;

pub struct MMU {
    mbc: Option<mbcType>,
    ram: ram::RAM,
    mmap: [u8; 0xffff]
}

impl MMU {
    pub fn new () -> MMU {
        debug!("Creating new MMU...");
        MMU {..Default::default()}
    }

    pub fn read_rom (&mut self, rom: &Vec<u8>) {
        self.mbc = MBCBuilder::from_rom(rom);
        match &self.mbc {
            Some(_mbc) => debug!("MBC created for ROM. Type: {}.", _mbc.get_type()),
            None => println!("No MBC could be created for this rom.") 
        }
    }

    /*fn set(&self, location: u32, value: u8) -> u8 {
        self.map[usize::try_from(location).unwrap()] = value;
        return value;
    }*/

    pub fn reset (&self) {

    }

    pub fn rb (&self, addr: u16) -> u8 {
        match &self.mbc {
            Some(_mbc) => {
                return match addr {
                    0x0000..=0x3FFF => _mbc.read(addr),
                    _ => self.mmap[addr as usize]
                }
            },
            None => panic!("MMU cannot read a byte without a mbc")
        };
    }

    pub fn wb (&mut self, addr: u16, value: u8) -> u8 {
        self.mmap[addr as usize] = value;
        return value;
    }
}

impl Default for MMU {
    fn default () -> MMU {
        MMU {
            mbc: None,
            ram: ram::RAM::new(),
            mmap: [0; 0xffff]
        }
    }
}