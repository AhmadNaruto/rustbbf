use std::fs::File;
use std::io::Read;
use std::path::Path;
use memmap2::Mmap;
use xxhash_rust::xxh3::{xxh3_64, xxh3_128};

use crate::bbf::types::{Header, Footer, Asset, Page, Section, Meta, VERSION, MAX_FORME_SIZE};

pub enum ReaderSource {
    Mmap(Mmap),
    Bytes(Vec<u8>),
}

impl AsRef<[u8]> for ReaderSource {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Mmap(m) => m.as_ref(),
            Self::Bytes(b) => b.as_ref(),
        }
    }
}

pub struct Reader {
    pub source: ReaderSource,
    pub header: Header,
    pub footer: Footer,
}

impl Reader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let meta = file.metadata().map_err(|e| e.to_string())?;
        let size = meta.len();
        if size < (Header::SIZE + Footer::SIZE) as u64 {
            return Err("File is too small to be a BBF container".to_string());
        }

        let source = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => ReaderSource::Mmap(mmap),
            Err(_) => {
                let mut f = file;
                let mut buffer = Vec::with_capacity(size as usize);
                f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                ReaderSource::Bytes(buffer)
            }
        };

        Self::new_from_source(source)
    }

    pub fn new(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() < Header::SIZE + Footer::SIZE {
            return Err("Byte array is too small to be a BBF container".to_string());
        }
        Self::new_from_source(ReaderSource::Bytes(bytes))
    }

    fn new_from_source(source: ReaderSource) -> Result<Self, String> {
        let bytes = source.as_ref();
        let header = Header::from_bytes(&bytes[0..Header::SIZE])
            .ok_or_else(|| "Failed to parse BBF header".to_string())?;

        if &header.magic != b"BBF3" {
            return Err("Invalid magic number".to_string());
        }

        if header.version != VERSION {
            return Err(format!("Unsupported BBF version: {}", header.version));
        }

        let footer_offset = header.footer_offset;
        if footer_offset + Footer::SIZE as u64 > bytes.len() as u64 {
            return Err("Footer offset goes out of file bounds".to_string());
        }

        let footer = Footer::from_bytes(&bytes[footer_offset as usize..footer_offset as usize + Footer::SIZE])
            .ok_or_else(|| "Failed to parse BBF footer".to_string())?;

        Ok(Self {
            source,
            header,
            footer,
        })
    }

    pub fn is_safe(&self, offset: u64, size: u64) -> bool {
        let file_size = self.source.as_ref().len() as u64;
        if let Some(end) = offset.checked_add(size) {
            end <= file_size
        } else {
            false
        }
    }

    pub fn get_asset(&self, index: u64) -> Result<Asset, String> {
        if index >= self.footer.asset_count {
            return Err(format!("Asset index {} out of bounds", index));
        }
        let offset = index
            .checked_mul(Asset::SIZE as u64)
            .and_then(|o| self.footer.asset_offset.checked_add(o))
            .ok_or_else(|| "Integer overflow calculating asset offset".to_string())?;

        if !self.is_safe(offset, Asset::SIZE as u64) {
            return Err("Asset entry goes out of file bounds".to_string());
        }
        let start = offset.try_into().map_err(|_| "Offset exceeds memory address space".to_string())?;
        Asset::from_bytes(&self.source.as_ref()[start..start + Asset::SIZE])
            .ok_or_else(|| "Failed to parse asset entry".to_string())
    }

    pub fn get_page(&self, index: u64) -> Result<Page, String> {
        if index >= self.footer.page_count {
            return Err(format!("Page index {} out of bounds", index));
        }
        let offset = index
            .checked_mul(Page::SIZE as u64)
            .and_then(|o| self.footer.page_offset.checked_add(o))
            .ok_or_else(|| "Integer overflow calculating page offset".to_string())?;

        if !self.is_safe(offset, Page::SIZE as u64) {
            return Err("Page entry goes out of file bounds".to_string());
        }
        let start = offset.try_into().map_err(|_| "Offset exceeds memory address space".to_string())?;
        Page::from_bytes(&self.source.as_ref()[start..start + Page::SIZE])
            .ok_or_else(|| "Failed to parse page entry".to_string())
    }

    pub fn get_section(&self, index: u64) -> Result<Section, String> {
        if index >= self.footer.section_count {
            return Err(format!("Section index {} out of bounds", index));
        }
        let offset = index
            .checked_mul(Section::SIZE as u64)
            .and_then(|o| self.footer.section_offset.checked_add(o))
            .ok_or_else(|| "Integer overflow calculating section offset".to_string())?;

        if !self.is_safe(offset, Section::SIZE as u64) {
            return Err("Section entry goes out of file bounds".to_string());
        }
        let start = offset.try_into().map_err(|_| "Offset exceeds memory address space".to_string())?;
        Section::from_bytes(&self.source.as_ref()[start..start + Section::SIZE])
            .ok_or_else(|| "Failed to parse section entry".to_string())
    }

    pub fn get_meta(&self, index: u64) -> Result<Meta, String> {
        if index >= self.footer.meta_count {
            return Err(format!("Metadata index {} out of bounds", index));
        }
        let offset = index
            .checked_mul(Meta::SIZE as u64)
            .and_then(|o| self.footer.meta_offset.checked_add(o))
            .ok_or_else(|| "Integer overflow calculating metadata offset".to_string())?;

        if !self.is_safe(offset, Meta::SIZE as u64) {
            return Err("Metadata entry goes out of file bounds".to_string());
        }
        let start = offset.try_into().map_err(|_| "Offset exceeds memory address space".to_string())?;
        Meta::from_bytes(&self.source.as_ref()[start..start + Meta::SIZE])
            .ok_or_else(|| "Failed to parse metadata entry".to_string())
    }

    pub fn get_string(&self, offset: u64) -> Result<String, String> {
        if offset == 0xFFFFFFFFFFFFFFFF {
            return Err("Null string offset".to_string());
        }
        let pool_offset = self.footer.string_pool_offset;
        let pool_size = self.footer.string_pool_size;
        if offset >= pool_size {
            return Err(format!("String offset {} out of pool size {}", offset, pool_size));
        }

        let abs_start = pool_offset
            .checked_add(offset)
            .ok_or_else(|| "Integer overflow calculating absolute string offset".to_string())?;
        let file_bytes = self.source.as_ref();
        
        let pool_end = pool_offset
            .checked_add(pool_size)
            .ok_or_else(|| "Integer overflow calculating string pool end".to_string())?;
        let abs_end_limit: usize = pool_end
            .try_into()
            .map_err(|_| "String pool end exceeds memory address space".to_string())?;

        let start: usize = abs_start
            .try_into()
            .map_err(|_| "Absolute string offset exceeds memory address space".to_string())?;

        if start >= file_bytes.len() {
            return Err("String offset starts out of file bounds".to_string());
        }

        let scan_end = std::cmp::min(start + MAX_FORME_SIZE, abs_end_limit);
        let slice = &file_bytes[start..scan_end];

        if let Some(null_pos) = slice.iter().position(|&b| b == 0) {
            let str_bytes = &slice[..null_pos];
            String::from_utf8(str_bytes.to_vec())
                .map_err(|e| format!("Invalid UTF-8 in string pool: {}", e))
        } else {
            Err("String is not null-terminated or exceeds max string length".to_string())
        }
    }

    pub fn get_asset_data(&self, asset: &Asset) -> Result<&[u8], String> {
        if !self.is_safe(asset.file_offset, asset.file_size) {
            return Err("Asset data range goes out of file bounds".to_string());
        }
        let start: usize = asset.file_offset
            .try_into()
            .map_err(|_| "Asset offset exceeds memory address space".to_string())?;
        let size: usize = asset.file_size
            .try_into()
            .map_err(|_| "Asset size exceeds memory address space".to_string())?;
        
        let end = start
            .checked_add(size)
            .ok_or_else(|| "Integer overflow calculating asset data end".to_string())?;
        Ok(&self.source.as_ref()[start..end])
    }

    pub fn verify_footer_hash(&self) -> bool {
        let index_start = match self.footer.asset_offset.try_into() {
            Ok(idx) => idx,
            Err(_) => return false,
        };
        let index_end_val = match self.footer.string_pool_offset.checked_add(self.footer.string_pool_size) {
            Some(end) => end,
            None => return false,
        };
        let index_end = match index_end_val.try_into() {
            Ok(idx) => idx,
            Err(_) => return false,
        };
        let bytes = self.source.as_ref();
        if index_start > bytes.len() || index_end > bytes.len() || index_start > index_end {
            return false;
        }
        let calculated = xxh3_64(&bytes[index_start..index_end]);
        calculated == self.footer.footer_hash
    }

    pub fn verify_asset_hash(&self, index: u64) -> bool {
        if let Ok(asset) = self.get_asset(index) {
            if let Ok(data) = self.get_asset_data(&asset) {
                let hash = xxh3_128(data);
                let low = hash as u64;
                let high = (hash >> 64) as u64;
                return asset.asset_hash[0] == low && asset.asset_hash[1] == high;
            }
        }
        false
    }
}
