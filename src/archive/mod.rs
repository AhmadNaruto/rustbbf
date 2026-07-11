use std::fs::File;
use std::path::Path;

pub mod cbz;
pub mod cbt;
pub mod cb7;

pub enum ArchiveFormat {
    Cbz,
    Cbt,
    Cb7,
    Cbr,
}

pub enum ArchiveWriterImpl {
    Cbz(cbz::CbzWriter),
    Cbt(cbt::CbtWriter),
    Cb7(cb7::Cb7Writer),
}

pub struct ArchiveBuilder {
    writer: ArchiveWriterImpl,
}

impl ArchiveBuilder {
    pub fn new<P: AsRef<Path>>(output_path: P, format: ArchiveFormat) -> Result<Self, String> {
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let writer = match format {
            ArchiveFormat::Cbz => ArchiveWriterImpl::Cbz(cbz::CbzWriter::new(file)?),
            ArchiveFormat::Cbt => ArchiveWriterImpl::Cbt(cbt::CbtWriter::new(file)?),
            ArchiveFormat::Cb7 => ArchiveWriterImpl::Cb7(cb7::Cb7Writer::new(file)?),
            ArchiveFormat::Cbr => {
                return Err("CBR (RAR) creation is not supported because the RAR compression encoder is proprietary and closed-source. Only extraction is supported in open source libraries.".to_string());
            }
        };
        Ok(Self { writer })
    }

    pub fn add_page<P: AsRef<Path>>(&mut self, file_path: P, name_in_archive: &str) -> Result<bool, String> {
        match &mut self.writer {
            ArchiveWriterImpl::Cbz(w) => w.add_page(file_path, name_in_archive),
            ArchiveWriterImpl::Cbt(w) => w.add_page(file_path, name_in_archive),
            ArchiveWriterImpl::Cb7(w) => w.add_page(file_path, name_in_archive),
        }
    }

    pub fn finalize(self) -> Result<(), String> {
        match self.writer {
            ArchiveWriterImpl::Cbz(w) => w.finalize(),
            ArchiveWriterImpl::Cbt(w) => w.finalize(),
            ArchiveWriterImpl::Cb7(w) => w.finalize(),
        }
    }
}
