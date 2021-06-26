const RAM_SIZE: usize = 8192;

pub struct RAM {
    data: [u8; RAM_SIZE]
}

impl RAM {
    pub fn new () -> RAM {
        debug!("Creating new RAM ({}KB)...", RAM_SIZE/1024);
        RAM {
            data: [0; RAM_SIZE]
        }
    }
}