pub fn from_file (file_name: &str) -> Vec<u8> {
    match std::fs::read(file_name) {
        Ok(bytes) => { 
            bytes
         }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                panic!("Could not open ROM file: {}. Please run again with appropriate permissions.\nError: {}", file_name, e);
            }
            panic!("{}", e);
        }
    }
}