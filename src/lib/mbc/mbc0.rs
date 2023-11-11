use super::MBC;

pub struct MBC0 {
    data: Vec<u8>,
    rom_size: u32
}

impl MBC0 {
    pub fn new () -> MBC0 {
        return MBC0 {
            data: [].to_vec(),
            rom_size: 0
        }
    }

    pub fn from_rom (rom: &Vec<u8>) -> MBC0 {
        debug!("Creating new MBC0 MBC...");
        return MBC0 {
            data: rom.clone(),
            rom_size: rom.len() as u32
        }
    }
}

impl MBC for MBC0 {
    fn read(&self, addr: u16) -> u8 {
        if (addr as u32) >= self.rom_size {
            panic!("MBC::MBC0 tried to read from unaccessible memory location")
        } 
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, byte: u8) {
        if (addr as u32) >= self.rom_size {
            panic!("MBC::MBC0 tried to write to unaccessible memory location")
        } 
        self.data[addr as usize] = byte
    }

    fn get_type(&self) -> &str {
        let s = "MBC0 (ROM only)";
        return &s;
    }
}