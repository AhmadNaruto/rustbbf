use std::fs::File;
use std::path::Path;
use tar::{Builder, Header};

pub struct CbtWriter {
    tar: Builder<File>,
}

impl CbtWriter {
    pub fn new(file: File) -> Result<Self, String> {
        Ok(Self {
            tar: Builder::new(file),
        })
    }

    pub fn add_page<P: AsRef<Path>>(&mut self, file_path: P, name_in_archive: &str) -> Result<bool, String> {
        let mut f = File::open(&file_path).map_err(|e| e.to_string())?;
        let metadata = f.metadata().map_err(|e| e.to_string())?;
        
        let mut header = Header::new_gnu();
        header.set_size(metadata.len());
        header.set_mode(0o644);
        
        self.tar.append_data(&mut header, name_in_archive, &mut f).map_err(|e| e.to_string())?;
        Ok(true)
    }

    pub fn finalize(mut self) -> Result<(), String> {
        self.tar.finish().map_err(|e| e.to_string())?;
        Ok(())
    }
}
