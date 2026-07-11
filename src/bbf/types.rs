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
    pub asset_hash: [u64; 2],
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
