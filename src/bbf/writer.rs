use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

use crate::bbf::types::{
    Header, Footer, Asset, Page, Section, Meta,
    BBF_PETRIFICATION_FLAG, BBF_VARIABLE_REAM_SIZE_FLAG,
    VERSION,
};

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
    
    let mut header_buf = [0u8; 64];
    in_file.read_exact(&mut header_buf).map_err(|e| e.to_string())?;
    let mut header = Header::from_bytes(&header_buf).ok_or("Invalid header")?;
    
    if &header.magic != b"BBF3" {
        return Err("Invalid magic".to_string());
    }
    
    if (header.flags & BBF_PETRIFICATION_FLAG) != 0 {
        return Err("File is already petrified".to_string());
    }
    
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
    
    if index_size > 16_000_000 {
        return Err("Index size exceeds safety limit of 16MB".to_string());
    }

    let mut index_bytes = vec![0u8; index_size as usize];
    in_file.seek(SeekFrom::Start(old_index_start)).map_err(|e| e.to_string())?;
    in_file.read_exact(&mut index_bytes).map_err(|e| e.to_string())?;
    
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
    
    footer.footer_hash = xxh3_64(&index_bytes);
    
    let mut out_file = File::create(output).map_err(|e| e.to_string())?;
    
    header.flags |= BBF_PETRIFICATION_FLAG;
    header.footer_offset = 64;
    
    out_file.write_all(&header.to_bytes()).map_err(|e| e.to_string())?;
    out_file.write_all(&footer.to_bytes()).map_err(|e| e.to_string())?;
    out_file.write_all(&index_bytes).map_err(|e| e.to_string())?;
    
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
