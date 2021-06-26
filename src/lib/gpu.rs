const VRAM_SIZE: usize = 8192;
pub struct GPU {
    data: [u8; VRAM_SIZE]
}

impl GPU {
    pub fn new () -> GPU {
        debug!("Creating new GPU ({}KB)...", VRAM_SIZE/1024);
        GPU {
            data: [0;VRAM_SIZE]
        }
    }
}