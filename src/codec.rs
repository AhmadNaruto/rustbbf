use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use memmap2::Mmap;
use xxhash_rust::xxh3::{xxh3_64, xxh3_128};

// Header Flags
pub const BBF_PETRIFICATION_FLAG: u32 = 0x00000001;
pub const BBF_VARIABLE_REAM_SIZE_FLAG: u32 = 0x00000002;

// Defaults
pub const DEFAULT_GUARD_ALIGNMENT: u8 = 12; // 2^12 = 4096
pub const DEFAULT_SMALL_REAM_THRESHOLD: u8 = 16; // 2^16 = 65536
pub const MAX_FORME_SIZE: usize = 2048;
pub const VERSION: u16 = 3;

#[derive(Debug, Clone)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: u16,
    pub header_len: u16,
    pub flags: u32,
    pub alignment: u8,
    pub ream_size: u8,
    pub reserved_extra: u16,
    pub footer_offset: u64,
}

impl Header {
    pub const SIZE: usize = 64;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        let header_len = u16::from_le_bytes(bytes[6..8].try_into().unwrap());
        let flags = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let alignment = bytes[12];
        let ream_size = bytes[13];
        let reserved_extra = u16::from_le_bytes(bytes[14..16].try_into().unwrap());
        let footer_offset = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        Some(Self {
            magic,
            version,
            header_len,
            flags,
            alignment,
            ream_size,
            reserved_extra,
            footer_offset,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.version.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.header_len.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.flags.to_le_bytes());
        bytes[12] = self.alignment;
        bytes[13] = self.ream_size;
        bytes[14..16].copy_from_slice(&self.reserved_extra.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.footer_offset.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Footer {
    pub asset_offset: u64,
    pub page_offset: u64,
    pub section_offset: u64,
    pub meta_offset: u64,
    pub expansion_offset: u64,
    pub string_pool_offset: u64,
    pub string_pool_size: u64,
    pub asset_count: u64,
    pub page_count: u64,
    pub section_count: u64,
    pub meta_count: u64,
    pub expansion_count: u64,
    pub flags: u32,
    pub footer_len: u8,
    pub padding: [u8; 3],
    pub footer_hash: u64,
}

impl Footer {
    pub const SIZE: usize = 256;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let asset_offset = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let page_offset = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let section_offset = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let meta_offset = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
        let expansion_offset = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
        let string_pool_offset = u64::from_le_bytes(bytes[40..48].try_into().unwrap());
        let string_pool_size = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
        let asset_count = u64::from_le_bytes(bytes[56..64].try_into().unwrap());
        let page_count = u64::from_le_bytes(bytes[64..72].try_into().unwrap());
        let section_count = u64::from_le_bytes(bytes[72..80].try_into().unwrap());
        let meta_count = u64::from_le_bytes(bytes[80..88].try_into().unwrap());
        let expansion_count = u64::from_le_bytes(bytes[88..96].try_into().unwrap());
        let flags = u32::from_le_bytes(bytes[96..100].try_into().unwrap());
        let footer_len = bytes[100];
        let padding = [bytes[101], bytes[102], bytes[103]];
        let footer_hash = u64::from_le_bytes(bytes[104..112].try_into().unwrap());
        Some(Self {
            asset_offset,
            page_offset,
            section_offset,
            meta_offset,
            expansion_offset,
            string_pool_offset,
            string_pool_size,
            asset_count,
            page_count,
            section_count,
            meta_count,
            expansion_count,
            flags,
            footer_len,
            padding,
            footer_hash,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.asset_offset.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.page_offset.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.section_offset.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.meta_offset.to_le_bytes());
        bytes[32..40].copy_from_slice(&self.expansion_offset.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.string_pool_offset.to_le_bytes());
        bytes[48..56].copy_from_slice(&self.string_pool_size.to_le_bytes());
        bytes[56..64].copy_from_slice(&self.asset_count.to_le_bytes());
        bytes[64..72].copy_from_slice(&self.page_count.to_le_bytes());
        bytes[72..80].copy_from_slice(&self.section_count.to_le_bytes());
        bytes[80..88].copy_from_slice(&self.meta_count.to_le_bytes());
        bytes[88..96].copy_from_slice(&self.expansion_count.to_le_bytes());
        bytes[96..100].copy_from_slice(&self.flags.to_le_bytes());
        bytes[100] = self.footer_len;
        bytes[101..104].copy_from_slice(&self.padding);
        bytes[104..112].copy_from_slice(&self.footer_hash.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub file_offset: u64,
    pub asset_hash: [u64; 2], // XXH3-128
    pub file_size: u64,
    pub flags: u32,
    pub reserved_value: u16,
    pub asset_type: u8,
}

impl Asset {
    pub const SIZE: usize = 48;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let file_offset = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let hash_low = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let hash_high = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let file_size = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
        let flags = u32::from_le_bytes(bytes[32..36].try_into().unwrap());
        let reserved_value = u16::from_le_bytes(bytes[36..38].try_into().unwrap());
        let asset_type = bytes[38];
        Some(Self {
            file_offset,
            asset_hash: [hash_low, hash_high],
            file_size,
            flags,
            reserved_value,
            asset_type,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.file_offset.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.asset_hash[0].to_le_bytes());
        bytes[16..24].copy_from_slice(&self.asset_hash[1].to_le_bytes());
        bytes[24..32].copy_from_slice(&self.file_size.to_le_bytes());
        bytes[32..36].copy_from_slice(&self.flags.to_le_bytes());
        bytes[36..38].copy_from_slice(&self.reserved_value.to_le_bytes());
        bytes[38] = self.asset_type;
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub asset_index: u64,
    pub flags: u32,
}

impl Page {
    pub const SIZE: usize = 16;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let asset_index = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let flags = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        Some(Self { asset_index, flags })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.asset_index.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.flags.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title_offset: u64,
    pub start_index: u64,
    pub parent_offset: u64,
}

impl Section {
    pub const SIZE: usize = 32;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let title_offset = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let start_index = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let parent_offset = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        Some(Self {
            title_offset,
            start_index,
            parent_offset,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.title_offset.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.start_index.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.parent_offset.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Meta {
    pub key_offset: u64,
    pub value_offset: u64,
    pub parent_offset: u64,
}

impl Meta {
    pub const SIZE: usize = 32;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let key_offset = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let value_offset = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let parent_offset = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        Some(Self {
            key_offset,
            value_offset,
            parent_offset,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.key_offset.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.value_offset.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.parent_offset.to_le_bytes());
        bytes
    }
}

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

        // Try memory mapping, fallback to reading bytes
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

pub struct Builder {
    file: File,
    current_offset: u64,
    alignment: u8,
    ream_size: u8,
    flags: u32,
    assets: Vec<Asset>,
    pages: Vec<Page>,
    sections: Vec<Section>,
    metadata: Vec<Meta>,
    string_pool_data: Vec<u8>,
    string_pool_map: HashMap<String, u64>,
    asset_lookup: HashMap<[u8; 16], u64>,
}

impl Builder {
    pub fn new<P: AsRef<Path>>(
        output_path: P,
        alignment: u8,
        ream_size: u8,
        flags: u32,
    ) -> Result<Self, String> {
        let mut file = File::create(output_path).map_err(|e| e.to_string())?;
        
        // Write blank header
        let blank_header = [0u8; Header::SIZE];
        file.write_all(&blank_header).map_err(|e| e.to_string())?;

        Ok(Self {
            file,
            current_offset: Header::SIZE as u64,
            alignment,
            ream_size,
            flags,
            assets: Vec::new(),
            pages: Vec::new(),
            sections: Vec::new(),
            metadata: Vec::new(),
            string_pool_data: Vec::new(),
            string_pool_map: HashMap::new(),
            asset_lookup: HashMap::new(),
        })
    }

    fn write_padding(&mut self, boundary: u64) -> Result<(), String> {
        let remainder = self.current_offset % boundary;
        if remainder == 0 {
            return Ok(());
        }
        let padding = boundary - remainder;
        let zeros = vec![0u8; padding as usize];
        self.file.write_all(&zeros).map_err(|e| e.to_string())?;
        self.current_offset += padding;
        Ok(())
    }

    fn detect_type<Q: AsRef<Path>>(&self, path: Q) -> u8 {
        if let Some(ext) = path.as_ref().extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_ascii_lowercase().as_str() {
                    "avif" => 0x01,
                    "png" => 0x02,
                    "webp" => 0x03,
                    "jxl" => 0x04,
                    "bmp" => 0x05,
                    "gif" => 0x07,
                    "tiff" | "tif" => 0x08,
                    "jpg" | "jpeg" => 0x09,
                    _ => 0x00,
                }
            } else {
                0x00
            }
        } else {
            0x00
        }
    }

    pub fn add_page<Q: AsRef<Path>>(
        &mut self,
        file_path: Q,
        page_flags: u32,
        asset_flags: u32,
    ) -> Result<bool, String> {
        let path = file_path.as_ref();
        let media_type = self.detect_type(path);
        
        let mut f = File::open(path).map_err(|e| e.to_string())?;
        let size = f.metadata().map_err(|e| e.to_string())?.len();

        // Stream hashing (O(1) Memory footprint)
        use xxhash_rust::xxh3::Xxh3;
        let mut hasher = Xxh3::new();
        let mut buffer = [0u8; 65536];
        loop {
            let bytes_read = f.read(&mut buffer).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let hash = hasher.digest128();
        let hash_low = hash as u64;
        let hash_high = (hash >> 64) as u64;
        let hash_key = {
            let mut key = [0u8; 16];
            key[0..8].copy_from_slice(&hash_low.to_le_bytes());
            key[8..16].copy_from_slice(&hash_high.to_le_bytes());
            key
        };

        if let Some(&asset_index) = self.asset_lookup.get(&hash_key) {
            self.pages.push(Page {
                asset_index,
                flags: page_flags,
            });
            return Ok(true);
        }

        // New asset
        let mut alignment_bytes = 1u64 << self.alignment;
        let threshold_bytes = 1u64 << self.ream_size;
        let variable_align = (self.flags & BBF_VARIABLE_REAM_SIZE_FLAG) != 0;

        if variable_align && size < threshold_bytes {
            alignment_bytes = 8;
        }

        self.write_padding(alignment_bytes)?;
        let asset_start_offset = self.current_offset;

        // Stream file copy (O(1) Memory footprint)
        f.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        loop {
            let bytes_read = f.read(&mut buffer).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break;
            }
            self.file.write_all(&buffer[..bytes_read]).map_err(|e| e.to_string())?;
        }
        self.current_offset = self.current_offset
            .checked_add(size)
            .ok_or_else(|| "Integer overflow calculating current offset".to_string())?;

        let asset_index = self.assets.len() as u64;
        self.assets.push(Asset {
            file_offset: asset_start_offset,
            asset_hash: [hash_low, hash_high],
            file_size: size,
            flags: asset_flags,
            reserved_value: 0,
            asset_type: media_type,
        });

        self.asset_lookup.insert(hash_key, asset_index);
        self.pages.push(Page {
            asset_index,
            flags: page_flags,
        });

        Ok(true)
    }

    fn add_string(&mut self, s: &str) -> u64 {
        if let Some(&offset) = self.string_pool_map.get(s) {
            offset
        } else {
            let offset = self.string_pool_data.len() as u64;
            self.string_pool_data.extend_from_slice(s.as_bytes());
            self.string_pool_data.push(0); // Null terminator
            self.string_pool_map.insert(s.to_string(), offset);
            offset
        }
    }

    pub fn add_meta(&mut self, key: &str, value: &str, parent: Option<&str>) -> bool {
        let key_offset = self.add_string(key);
        let value_offset = self.add_string(value);
        let parent_offset = if let Some(p) = parent {
            self.add_string(p)
        } else {
            0xFFFFFFFFFFFFFFFF
        };
        self.metadata.push(Meta {
            key_offset,
            value_offset,
            parent_offset,
        });
        true
    }

    pub fn add_section(&mut self, name: &str, start_index: u64, parent: Option<&str>) -> bool {
        if start_index > self.pages.len() as u64 {
            return false;
        }
        let title_offset = self.add_string(name);
        let parent_offset = if let Some(p) = parent {
            self.add_string(p)
        } else {
            0xFFFFFFFFFFFFFFFF
        };
        self.sections.push(Section {
            title_offset,
            start_index,
            parent_offset,
        });
        true
    }

    pub fn finalize(mut self) -> Result<(), String> {
        if self.assets.is_empty() {
            return Err("No assets to finalize".to_string());
        }

        // Serialize all indexes to compute the footer hash
        let mut index_bytes = Vec::new();
        for asset in &self.assets {
            index_bytes.extend_from_slice(&asset.to_bytes());
        }
        let asset_offset = self.current_offset;
        let asset_bytes_len = index_bytes.len() as u64;

        let page_offset = asset_offset + asset_bytes_len;
        for page in &self.pages {
            index_bytes.extend_from_slice(&page.to_bytes());
        }
        let page_bytes_len = index_bytes.len() as u64 - asset_bytes_len;

        let section_offset = page_offset + page_bytes_len;
        for section in &self.sections {
            index_bytes.extend_from_slice(&section.to_bytes());
        }
        let section_bytes_len = index_bytes.len() as u64 - asset_bytes_len - page_bytes_len;

        let meta_offset = section_offset + section_bytes_len;
        for meta in &self.metadata {
            index_bytes.extend_from_slice(&meta.to_bytes());
        }
        let meta_bytes_len = index_bytes.len() as u64 - asset_bytes_len - page_bytes_len - section_bytes_len;

        let string_pool_offset = meta_offset + meta_bytes_len;
        index_bytes.extend_from_slice(&self.string_pool_data);
        let string_pool_size = self.string_pool_data.len() as u64;

        let footer_hash = xxh3_64(&index_bytes);

        let footer = Footer {
            asset_offset,
            page_offset,
            section_offset,
            meta_offset,
            expansion_offset: 0,
            string_pool_offset,
            string_pool_size,
            asset_count: self.assets.len() as u64,
            page_count: self.pages.len() as u64,
            section_count: self.sections.len() as u64,
            meta_count: self.metadata.len() as u64,
            expansion_count: 0,
            flags: 0,
            footer_len: Footer::SIZE as u8,
            padding: [0, 0, 0],
            footer_hash,
        };

        // Write index region
        self.file.write_all(&index_bytes).map_err(|e| e.to_string())?;
        self.current_offset += index_bytes.len() as u64;

        // Write footer
        let footer_offset = self.current_offset;
        self.file.write_all(&footer.to_bytes()).map_err(|e| e.to_string())?;

        // Go back to write Header
        let header = Header {
            magic: *b"BBF3",
            version: VERSION,
            header_len: Header::SIZE as u16,
            flags: self.flags,
            alignment: self.alignment,
            ream_size: self.ream_size,
            reserved_extra: 0,
            footer_offset,
        };

        self.file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        self.file.write_all(&header.to_bytes()).map_err(|e| e.to_string())?;

        Ok(())
    }
}

pub fn petrify_file<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<(), String> {
    let mut in_file = File::open(input).map_err(|e| e.to_string())?;
    
    // Read header
    let mut header_buf = [0u8; 64];
    in_file.read_exact(&mut header_buf).map_err(|e| e.to_string())?;
    let mut header = Header::from_bytes(&header_buf).ok_or("Invalid header")?;
    
    if &header.magic != b"BBF3" {
        return Err("Invalid magic".to_string());
    }
    
    if (header.flags & BBF_PETRIFICATION_FLAG) != 0 {
        return Err("File is already petrified".to_string());
    }
    
    // Read footer
    in_file.seek(SeekFrom::Start(header.footer_offset)).map_err(|e| e.to_string())?;
    let mut footer_buf = [0u8; 256];
    in_file.read_exact(&mut footer_buf).map_err(|e| e.to_string())?;
    let mut footer = Footer::from_bytes(&footer_buf).ok_or("Invalid footer")?;
    
    let old_index_start = footer.asset_offset;
    let index_size = header.footer_offset
        .checked_sub(old_index_start)
        .ok_or_else(|| "Invalid footer offsets".to_string())?;
    let data_size = old_index_start
        .checked_sub(64)
        .ok_or_else(|| "Invalid asset table offset".to_string())?;
    
    // Safety check for index size to prevent huge memory allocation
    if index_size > 16_000_000 {
        return Err("Index size exceeds safety limit of 16MB".to_string());
    }

    // Read index bytes
    in_file.seek(SeekFrom::Start(old_index_start)).map_err(|e| e.to_string())?;
    let mut index_bytes = vec![0u8; index_size as usize];
    in_file.read_exact(&mut index_bytes).map_err(|e| e.to_string())?;
    
    // Update footer offsets
    let new_index_start = 320u64; // 64 + 256
    let shift_index = new_index_start as i64 - old_index_start as i64;
    
    footer.asset_offset = (footer.asset_offset as i64 + shift_index) as u64;
    footer.page_offset = (footer.page_offset as i64 + shift_index) as u64;
    footer.section_offset = (footer.section_offset as i64 + shift_index) as u64;
    footer.meta_offset = (footer.meta_offset as i64 + shift_index) as u64;
    if footer.expansion_offset != 0 {
        footer.expansion_offset = (footer.expansion_offset as i64 + shift_index) as u64;
    }
    footer.string_pool_offset = (footer.string_pool_offset as i64 + shift_index) as u64;
    
    // Update Asset entries inside index_bytes
    let asset_count = footer.asset_count as usize;
    let asset_table_offset_in_index = (footer.asset_offset - new_index_start) as usize;
    
    let new_data_start = new_index_start + index_size;
    let shift_data = new_data_start as i64 - 64i64;
    
    for i in 0..asset_count {
        let entry_offset = asset_table_offset_in_index + i * 48;
        if entry_offset + 48 <= index_bytes.len() {
            let mut asset = Asset::from_bytes(&index_bytes[entry_offset..entry_offset + 48]).ok_or("Invalid asset struct")?;
            asset.file_offset = (asset.file_offset as i64 + shift_data) as u64;
            index_bytes[entry_offset..entry_offset + 48].copy_from_slice(&asset.to_bytes());
        }
    }
    
    // Recalculate footer hash
    footer.footer_hash = xxh3_64(&index_bytes);
    
    // Write petrified file
    let mut out_file = File::create(output).map_err(|e| e.to_string())?;
    
    // Update Header
    header.flags |= BBF_PETRIFICATION_FLAG;
    header.footer_offset = 64;
    
    out_file.write_all(&header.to_bytes()).map_err(|e| e.to_string())?;
    out_file.write_all(&footer.to_bytes()).map_err(|e| e.to_string())?;
    out_file.write_all(&index_bytes).map_err(|e| e.to_string())?;
    
    // Copy data region block by block (O(1) memory)
    in_file.seek(SeekFrom::Start(64)).map_err(|e| e.to_string())?;
    let mut buffer = vec![0u8; 65536];
    let mut remaining = data_size;
    while remaining > 0 {
        let chunk = std::cmp::min(remaining, buffer.len() as u64) as usize;
        in_file.read_exact(&mut buffer[..chunk]).map_err(|e| e.to_string())?;
        out_file.write_all(&buffer[..chunk]).map_err(|e| e.to_string())?;
        remaining -= chunk as u64;
    }
    
    Ok(())
}
