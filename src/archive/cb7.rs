use std::fs::File;
use std::path::Path;
use sevenz_rust::{SevenZWriter, SevenZArchiveEntry};

pub struct Cb7Writer {
    sz: SevenZWriter<File>,
}

impl Cb7Writer {
    pub fn new(file: File) -> Result<Self, String> {
        Ok(Self {
            sz: SevenZWriter::new(file).map_err(|e| e.to_string())?,
        })
    }

    pub fn add_page<P: AsRef<Path>>(&mut self, file_path: P, name_in_archive: &str) -> Result<bool, String> {
        let path = file_path.as_ref();
        let f = File::open(path).map_err(|e| e.to_string())?;
        
        let entry = SevenZArchiveEntry::from_path(path, name_in_archive.to_string());
        self.sz.push_archive_entry(entry, Some(f)).map_err(|e| e.to_string())?;
        Ok(true)
    }

    pub fn finalize(self) -> Result<(), String> {
        self.sz.finish().map_err(|e| e.to_string())?;
        Ok(())
    }
}
