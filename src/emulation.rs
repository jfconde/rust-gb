use log::{debug};
use crate::lib::cpu;

pub struct Emulation {
    rom_data: Vec<u8>,
    cpu: cpu::CPU
}

impl Emulation {
    pub fn from_rom(rom: Vec<u8>) -> Emulation {
        debug!("[EMU] Creating new emulation from ROM with size: {} bytes ({} KB)...", rom.len(), (rom.len() / 1024));
        
        let mut emulation = Emulation {
            rom_data: rom,
            cpu: cpu::CPU::new()
        };

        emulation.cpu.read_rom(&emulation.rom_data);
        emulation
    }

    pub fn start(&mut self) {
        self.cpu.start()
    }
}