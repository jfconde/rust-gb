#[macro_use] extern crate log;
mod lib;
mod emulation;

fn main() {
    env_logger::init();

    let rom_data: Vec<u8> = lib::rom::from_file("./roms/tetris.gb");
    let mut e = emulation::Emulation::from_rom(rom_data);
    e.start();
}
