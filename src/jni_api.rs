use jni::JNIEnv;
use jni::objects::{JClass, JString, JByteBuffer};
use jni::sys::{jboolean, jint, jlong, jobject, jstring, jbyteArray};
use crate::bbf::{Reader, Builder};

unsafe fn get_reader<'a>(handle: jlong) -> Option<&'a Reader> {
    if handle == 0 {
        None
    } else {
        Some(&*(handle as *const Reader))
    }
}

unsafe fn get_mut_builder<'a>(handle: jlong) -> Option<&'a mut Builder> {
    if handle == 0 {
        None
    } else {
        Some(&mut *(handle as *mut Builder))
    }
}

unsafe fn jstring_to_string(env: &mut JNIEnv, j_str: jstring) -> String {
    if j_str.is_null() {
        return String::new();
    }
    env.get_string(&JString::from_raw(j_str))
        .map(|s| s.into())
        .unwrap_or_default()
}

unsafe fn jstring_to_option_string(env: &mut JNIEnv, j_str: jstring) -> Option<String> {
    if j_str.is_null() {
        None
    } else {
        let s: String = env.get_string(&JString::from_raw(j_str))
            .map(|s| s.into())
            .unwrap_or_default();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

fn to_jstring(env: &mut JNIEnv, res: Result<String, String>) -> jstring {
    match res {
        Ok(s) => match env.new_string(s) {
            Ok(js) => js.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

// BBFReader Native Interface

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_openNative(
    mut env: JNIEnv,
    _class: JClass,
    file_path: jstring,
) -> jlong {
    let path = jstring_to_string(&mut env, file_path);
    match Reader::open(path) {
        Ok(reader) => Box::into_raw(Box::new(reader)) as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_openBytesNative(
    env: JNIEnv,
    _class: JClass,
    bytes: jbyteArray,
) -> jlong {
    if bytes.is_null() {
        return 0;
    }
    let j_arr = jni::objects::JByteArray::from_raw(bytes);
    match env.convert_byte_array(&j_arr) {
        Ok(rust_bytes) => match Reader::new(rust_bytes) {
            Ok(reader) => Box::into_raw(Box::new(reader)) as jlong,
            Err(_) => 0,
        },
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_closeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        let _ = Box::from_raw(handle as *mut Reader);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getVersion(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    match get_reader(handle) {
        Some(reader) => reader.header.version as jint,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getHeaderFlags(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    match get_reader(handle) {
        Some(reader) => reader.header.flags as jint,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAlignment(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    match get_reader(handle) {
        Some(reader) => reader.header.alignment as jint,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getReamSize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    match get_reader(handle) {
        Some(reader) => reader.header.ream_size as jint,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAssetCount(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    match get_reader(handle) {
        Some(reader) => reader.footer.asset_count as jlong,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getPageCount(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    match get_reader(handle) {
        Some(reader) => reader.footer.page_count as jlong,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getSectionCount(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    match get_reader(handle) {
        Some(reader) => reader.footer.section_count as jlong,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getMetaCount(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jlong {
    match get_reader(handle) {
        Some(reader) => reader.footer.meta_count as jlong,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getMetaKey(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jstring {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    let res = reader.get_meta(index as u64)
        .and_then(|m| reader.get_string(m.key_offset));
    to_jstring(&mut env, res)
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getMetaValue(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jstring {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    let res = reader.get_meta(index as u64)
        .and_then(|m| reader.get_string(m.value_offset));
    to_jstring(&mut env, res)
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getMetaParent(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jstring {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    let res = reader.get_meta(index as u64).and_then(|m| {
        if m.parent_offset == 0xFFFFFFFFFFFFFFFF {
            Ok(String::new())
        } else {
            reader.get_string(m.parent_offset)
        }
    });
    to_jstring(&mut env, res)
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getSectionTitle(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jstring {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    let res = reader.get_section(index as u64)
        .and_then(|s| reader.get_string(s.title_offset));
    to_jstring(&mut env, res)
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getSectionStartIndex(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jlong {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return -1,
    };
    match reader.get_section(index as u64) {
        Ok(s) => s.start_index as jlong,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getSectionParent(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jstring {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    let res = reader.get_section(index as u64).and_then(|s| {
        if s.parent_offset == 0xFFFFFFFFFFFFFFFF {
            Ok(String::new())
        } else {
            reader.get_string(s.parent_offset)
        }
    });
    to_jstring(&mut env, res)
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getPageAssetIndex(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jlong {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return -1,
    };
    match reader.get_page(index as u64) {
        Ok(p) => p.asset_index as jlong,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getPageFlags(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jint {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return 0,
    };
    match reader.get_page(index as u64) {
        Ok(p) => p.flags as jint,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAssetFileOffset(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jlong {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return -1,
    };
    match reader.get_asset(index as u64) {
        Ok(a) => a.file_offset as jlong,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAssetFileSize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jlong {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return -1,
    };
    match reader.get_asset(index as u64) {
        Ok(a) => a.file_size as jlong,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAssetFlags(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jint {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return 0,
    };
    match reader.get_asset(index as u64) {
        Ok(a) => a.flags as jint,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAssetType(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jint {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return 0,
    };
    match reader.get_asset(index as u64) {
        Ok(a) => a.asset_type as jint,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAssetHash(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jbyteArray {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    match reader.get_asset(index as u64) {
        Ok(a) => {
            let mut hash_bytes = [0u8; 16];
            hash_bytes[0..8].copy_from_slice(&a.asset_hash[0].to_le_bytes());
            hash_bytes[8..16].copy_from_slice(&a.asset_hash[1].to_le_bytes());
            match env.byte_array_from_slice(&hash_bytes) {
                Ok(arr) => arr.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_getAssetData(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jbyteArray {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return std::ptr::null_mut(),
    };
    match reader.get_asset(index as u64) {
        Ok(asset) => match reader.get_asset_data(&asset) {
            Ok(data) => {
                match env.byte_array_from_slice(data) {
                    Ok(arr) => arr.into_raw(),
                    Err(_) => std::ptr::null_mut(),
                }
            }
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_readAssetDataDirect(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
    byte_buffer: jobject,
    offset: jint,
    len: jint,
) -> jint {
    let reader = match get_reader(handle) {
        Some(r) => r,
        None => return -1,
    };
    let asset = match reader.get_asset(index as u64) {
        Ok(a) => a,
        Err(_) => return -1,
    };
    let data = match reader.get_asset_data(&asset) {
        Ok(d) => d,
        Err(_) => return -2,
    };
    
    let j_buf = JByteBuffer::from_raw(byte_buffer);
    let buf_ptr = match env.get_direct_buffer_address(&j_buf) {
        Ok(ptr) => ptr,
        Err(_) => return -3,
    };
    let buf_cap = match env.get_direct_buffer_capacity(&j_buf) {
        Ok(cap) => cap,
        Err(_) => return -4,
    };
    
    if offset < 0 || len < 0 {
        return -5;
    }
    
    let dest_offset = offset as usize;
    let read_len = std::cmp::min(len as usize, data.len());
    
    if dest_offset + read_len > buf_cap {
        return -6;
    }
    
    let dest_slice = std::slice::from_raw_parts_mut(buf_ptr.add(dest_offset), read_len);
    dest_slice.copy_from_slice(&data[..read_len]);
    
    read_len as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_verifyFooterHash(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    match get_reader(handle) {
        Some(reader) => reader.verify_footer_hash() as jboolean,
        None => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFReader_verifyAssetHash(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    index: jint,
) -> jboolean {
    match get_reader(handle) {
        Some(reader) => reader.verify_asset_hash(index as u64) as jboolean,
        None => 0,
    }
}


// BBFBuilder Native Interface

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFBuilder_createNative(
    mut env: JNIEnv,
    _class: JClass,
    output_path: jstring,
    alignment: jint,
    ream_size: jint,
    flags: jint,
) -> jlong {
    let path = jstring_to_string(&mut env, output_path);
    match Builder::new(path, alignment as u8, ream_size as u8, flags as u32) {
        Ok(builder) => Box::into_raw(Box::new(builder)) as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFBuilder_closeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        let _ = Box::from_raw(handle as *mut Builder);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFBuilder_addPage(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    file_path: jstring,
    page_flags: jint,
    asset_flags: jint,
) -> jboolean {
    let builder = match get_mut_builder(handle) {
        Some(b) => b,
        None => return 0,
    };
    let path = jstring_to_string(&mut env, file_path);
    match builder.add_page(path, page_flags as u32, asset_flags as u32) {
        Ok(success) => success as jboolean,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFBuilder_addMeta(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    key: jstring,
    value: jstring,
    parent: jstring,
) -> jboolean {
    let builder = match get_mut_builder(handle) {
        Some(b) => b,
        None => return 0,
    };
    let key_str = jstring_to_string(&mut env, key);
    let val_str = jstring_to_string(&mut env, value);
    let parent_opt = jstring_to_option_string(&mut env, parent);

    builder.add_meta(&key_str, &val_str, parent_opt.as_deref()) as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFBuilder_addSection(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    name: jstring,
    start_index: jlong,
    parent: jstring,
) -> jboolean {
    let builder = match get_mut_builder(handle) {
        Some(b) => b,
        None => return 0,
    };
    let name_str = jstring_to_string(&mut env, name);
    let parent_opt = jstring_to_option_string(&mut env, parent);

    builder.add_section(&name_str, start_index as u64, parent_opt.as_deref()) as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFBuilder_finalizeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    if handle == 0 {
        return 0;
    }
    let builder = Box::from_raw(handle as *mut Builder);
    match builder.finalize() {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_BBFBuilder_petrifyNative(
    mut env: JNIEnv,
    _class: JClass,
    input_path: jstring,
    output_path: jstring,
) -> jboolean {
    let in_str = jstring_to_string(&mut env, input_path);
    let out_str = jstring_to_string(&mut env, output_path);
        
    match crate::bbf::petrify_file(in_str, out_str) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

// ArchiveBuilder JNI Native Interface

use crate::archive::{ArchiveBuilder, ArchiveFormat};

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_ArchiveBuilder_createNative(
    mut env: JNIEnv,
    _class: JClass,
    output_path: jstring,
    format: jint,
) -> jlong {
    let path = jstring_to_string(&mut env, output_path);
    let fmt = match format {
        0 => ArchiveFormat::Cbz,
        1 => ArchiveFormat::Cbt,
        2 => ArchiveFormat::Cb7,
        _ => ArchiveFormat::Cbr,
    };
    match ArchiveBuilder::new(path, fmt) {
        Ok(builder) => Box::into_raw(Box::new(builder)) as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_ArchiveBuilder_closeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        let _ = Box::from_raw(handle as *mut ArchiveBuilder);
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_ArchiveBuilder_addPage(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    file_path: jstring,
    name_in_archive: jstring,
) -> jboolean {
    let builder = if handle == 0 {
        return 0;
    } else {
        &mut *(handle as *mut ArchiveBuilder)
    };
    let path = jstring_to_string(&mut env, file_path);
    let entry_name = jstring_to_string(&mut env, name_in_archive);
    match builder.add_page(path, &entry_name) {
        Ok(success) => success as jboolean,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_github_anaruto_libbbf_ArchiveBuilder_finalizeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    if handle == 0 {
        return 0;
    }
    let builder = Box::from_raw(handle as *mut ArchiveBuilder);
    match builder.finalize() {
        Ok(_) => 1,
        Err(_) => 0,
    }
}
