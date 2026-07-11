use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub struct CbzWriter {
    zip: ZipWriter<File>,
}

impl CbzWriter {
    pub fn new(file: File) -> Result<Self, String> {
        Ok(Self {
            zip: ZipWriter::new(file),
        })
    }

    pub fn add_page<P: AsRef<Path>>(&mut self, file_path: P, name_in_archive: &str) -> Result<bool, String> {
        let mut f = File::open(file_path).map_err(|e| e.to_string())?;
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
            
        self.zip.start_file(name_in_archive, options).map_err(|e| e.to_string())?;
        
        let mut buffer = [0u8; 65536];
        loop {
            let bytes_read = f.read(&mut buffer).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break;
            }
            self.zip.write_all(&buffer[..bytes_read]).map_err(|e| e.to_string())?;
        }
        Ok(true)
    }

    pub fn finalize(self) -> Result<(), String> {
        self.zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    }
}
