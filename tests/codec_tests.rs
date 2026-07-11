use std::fs::File;
use std::io::{Write, Read};
use bbf::bbf::{Builder, Reader, BBF_VARIABLE_REAM_SIZE_FLAG};

#[test]
fn test_build_read_petrify_roundtrip() {
    // 1. Create a temporary directory for our test files
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    
    // Create dummy files
    let file1_path = dir.path().join("image1.png");
    let mut file1 = File::create(&file1_path).unwrap();
    file1.write_all(b"PNG_FAKE_IMAGE_DATA_1").unwrap();
    
    let file2_path = dir.path().join("image2.png");
    let mut file2 = File::create(&file2_path).unwrap();
    file2.write_all(b"PNG_FAKE_IMAGE_DATA_2_DIFFERENT").unwrap();
    
    // This file will have identical content to file1 to test deduplication
    let file3_path = dir.path().join("image3.png");
    let mut file3 = File::create(&file3_path).unwrap();
    file3.write_all(b"PNG_FAKE_IMAGE_DATA_1").unwrap();

    let output_path = dir.path().join("test_container.bbf");
    
    // 2. Build container
    {
        let mut builder = Builder::new(
            &output_path,
            12, // alignment exponent: 2^12 = 4096
            16, // ream exponent: 2^16 = 65536
            BBF_VARIABLE_REAM_SIZE_FLAG,
        ).expect("Failed to create builder");
        
        assert!(builder.add_page(&file1_path, 0, 0).unwrap());
        assert!(builder.add_page(&file2_path, 0, 0).unwrap());
        assert!(builder.add_page(&file3_path, 0, 0).unwrap()); // should deduplicate to asset index 0
        
        assert!(builder.add_meta("title", "Test Book", None));
        assert!(builder.add_meta("author", "Rust Port", Some("credits")));
        
        assert!(builder.add_section("Chapter 1", 0, None));
        assert!(builder.add_section("Chapter 2", 1, Some("Chapter 1")));
        
        builder.finalize().expect("Failed to finalize container");
    }
    
    // 3. Read container (default layout)
    {
        let reader = Reader::open(&output_path).expect("Failed to open reader");
        
        // Assert header info
        assert_eq!(reader.header.version, 3);
        assert_eq!(reader.header.alignment, 12);
        assert_eq!(reader.header.ream_size, 16);
        assert_eq!(reader.header.flags & 1, 0); // Not petrified
        
        // Assert counts
        assert_eq!(reader.footer.asset_count, 2); // 3 pages, but 2 unique assets (1 and 3 are identical)
        assert_eq!(reader.footer.page_count, 3);
        assert_eq!(reader.footer.meta_count, 2);
        assert_eq!(reader.footer.section_count, 2);
        
        // Verify footer hash
        assert!(reader.verify_footer_hash());
        
        // Verify pages and asset links
        let page0 = reader.get_page(0).unwrap();
        let page1 = reader.get_page(1).unwrap();
        let page2 = reader.get_page(2).unwrap();
        
        assert_eq!(page0.asset_index, 0);
        assert_eq!(page1.asset_index, 1);
        assert_eq!(page2.asset_index, 0); // deduplicated to index 0
        
        // Verify asset contents
        let asset0 = reader.get_asset(0).unwrap();
        let asset0_data = reader.get_asset_data(&asset0).unwrap();
        assert_eq!(asset0_data, b"PNG_FAKE_IMAGE_DATA_1");
        assert_eq!(asset0.asset_type, 0x02); // PNG type
        assert!(reader.verify_asset_hash(0));
        
        let asset1 = reader.get_asset(1).unwrap();
        let asset1_data = reader.get_asset_data(&asset1).unwrap();
        assert_eq!(asset1_data, b"PNG_FAKE_IMAGE_DATA_2_DIFFERENT");
        assert_eq!(asset1.asset_type, 0x02); // PNG type
        assert!(reader.verify_asset_hash(1));
        
        // Verify metadata
        let meta0 = reader.get_meta(0).unwrap();
        assert_eq!(reader.get_string(meta0.key_offset).unwrap(), "title");
        assert_eq!(reader.get_string(meta0.value_offset).unwrap(), "Test Book");
        assert_eq!(meta0.parent_offset, 0xFFFFFFFFFFFFFFFF);
        
        let meta1 = reader.get_meta(1).unwrap();
        assert_eq!(reader.get_string(meta1.key_offset).unwrap(), "author");
        assert_eq!(reader.get_string(meta1.value_offset).unwrap(), "Rust Port");
        assert_eq!(reader.get_string(meta1.parent_offset).unwrap(), "credits");
        
        // Verify sections
        let sec0 = reader.get_section(0).unwrap();
        assert_eq!(reader.get_string(sec0.title_offset).unwrap(), "Chapter 1");
        assert_eq!(sec0.start_index, 0);
        assert_eq!(sec0.parent_offset, 0xFFFFFFFFFFFFFFFF);
        
        let sec1 = reader.get_section(1).unwrap();
        assert_eq!(reader.get_string(sec1.title_offset).unwrap(), "Chapter 2");
        assert_eq!(sec1.start_index, 1);
        assert_eq!(reader.get_string(sec1.parent_offset).unwrap(), "Chapter 1");
    }

    // 4. Petrify container
    let petrified_path = dir.path().join("petrified_container.bbf");
    bbf::bbf::petrify_file(&output_path, &petrified_path).expect("Failed to petrify");

    // 5. Read petrified container
    {
        let reader = Reader::open(&petrified_path).expect("Failed to open petrified reader");
        
        // Assert header info
        assert_eq!(reader.header.version, 3);
        assert_eq!(reader.header.alignment, 12);
        assert_eq!(reader.header.ream_size, 16);
        assert_ne!(reader.header.flags & 1, 0); // Petrified flag is set!
        
        // Assert same counts
        assert_eq!(reader.footer.asset_count, 2);
        assert_eq!(reader.footer.page_count, 3);
        assert_eq!(reader.footer.meta_count, 2);
        assert_eq!(reader.footer.section_count, 2);
        
        // Verify footer hash
        assert!(reader.verify_footer_hash());
        
        // Verify asset contents are still correct (they were offset-shifted!)
        let asset0 = reader.get_asset(0).unwrap();
        let asset0_data = reader.get_asset_data(&asset0).unwrap();
        assert_eq!(asset0_data, b"PNG_FAKE_IMAGE_DATA_1");
        assert!(reader.verify_asset_hash(0));
        
        let asset1 = reader.get_asset(1).unwrap();
        let asset1_data = reader.get_asset_data(&asset1).unwrap();
        assert_eq!(asset1_data, b"PNG_FAKE_IMAGE_DATA_2_DIFFERENT");
        assert!(reader.verify_asset_hash(1));
    }
}

#[test]
fn test_logical_page_order_preservation() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    
    // Create 3 distinct "real" images with different content to prevent deduplication
    let path_b = dir.path().join("img_b.png");
    let mut file_b = File::create(&path_b).unwrap();
    file_b.write_all(b"IMAGE_CONTENT_B").unwrap();
    
    let path_c = dir.path().join("img_c.png");
    let mut file_c = File::create(&path_c).unwrap();
    file_c.write_all(b"IMAGE_CONTENT_C").unwrap();
    
    let path_a = dir.path().join("img_a.png");
    let mut file_a = File::create(&path_a).unwrap();
    file_a.write_all(b"IMAGE_CONTENT_A").unwrap();
    
    let container_path = dir.path().join("ordered.bbf");
    
    // Mux them in B, C, A order
    {
        let mut builder = Builder::new(&container_path, 12, 16, 0).unwrap();
        builder.add_page(&path_b, 0, 0).unwrap();
        builder.add_page(&path_c, 0, 0).unwrap();
        builder.add_page(&path_a, 0, 0).unwrap();
        builder.finalize().unwrap();
    }
    
    // Read back and check the logical page sequence
    let reader = Reader::open(&container_path).unwrap();
    assert_eq!(reader.footer.page_count, 3);
    assert_eq!(reader.footer.asset_count, 3);
    
    // Check pages (logical order)
    let p0 = reader.get_page(0).unwrap();
    let p1 = reader.get_page(1).unwrap();
    let p2 = reader.get_page(2).unwrap();
    
    let asset0 = reader.get_asset(p0.asset_index).unwrap();
    let asset1 = reader.get_asset(p1.asset_index).unwrap();
    let asset2 = reader.get_asset(p2.asset_index).unwrap();
    
    let data0 = reader.get_asset_data(&asset0).unwrap();
    let data1 = reader.get_asset_data(&asset1).unwrap();
    let data2 = reader.get_asset_data(&asset2).unwrap();
    
    // Assert logical order is preserved exactly as inserted (B, C, A)
    assert_eq!(data0, b"IMAGE_CONTENT_B");
    assert_eq!(data1, b"IMAGE_CONTENT_C");
    assert_eq!(data2, b"IMAGE_CONTENT_A");
}

#[test]
fn test_archive_builders_cbz_cbt_cb7() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    
    let img_a = dir.path().join("a.png");
    File::create(&img_a).unwrap().write_all(b"A_DATA").unwrap();
    let img_b = dir.path().join("b.png");
    File::create(&img_b).unwrap().write_all(b"B_DATA").unwrap();

    // 1. Test CBZ (Zip)
    let cbz_path = dir.path().join("test.cbz");
    {
        let mut builder = bbf::archive::ArchiveBuilder::new(&cbz_path, bbf::archive::ArchiveFormat::Cbz).unwrap();
        builder.add_page(&img_a, "a.png").unwrap();
        builder.add_page(&img_b, "b.png").unwrap();
        builder.finalize().unwrap();
    }
    assert!(cbz_path.exists());

    let zip_file = File::open(&cbz_path).unwrap();
    let mut zip_archive = zip::ZipArchive::new(zip_file).unwrap();
    assert_eq!(zip_archive.len(), 2);
    
    let mut a_entry = zip_archive.by_name("a.png").unwrap();
    let mut a_data = Vec::new();
    a_entry.read_to_end(&mut a_data).unwrap();
    assert_eq!(a_data, b"A_DATA");

    // 2. Test CBT (Tar)
    let cbt_path = dir.path().join("test.cbt");
    {
        let mut builder = bbf::archive::ArchiveBuilder::new(&cbt_path, bbf::archive::ArchiveFormat::Cbt).unwrap();
        builder.add_page(&img_a, "a.png").unwrap();
        builder.add_page(&img_b, "b.png").unwrap();
        builder.finalize().unwrap();
    }
    assert!(cbt_path.exists());

    let tar_file = File::open(&cbt_path).unwrap();
    let mut tar_archive = tar::Archive::new(tar_file);
    let entries: Vec<_> = tar_archive.entries().unwrap().map(|e| e.unwrap()).collect();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path().unwrap().to_str().unwrap(), "a.png");

    // 3. Test CB7 (7z)
    let cb7_path = dir.path().join("test.cb7");
    {
        let mut builder = bbf::archive::ArchiveBuilder::new(&cb7_path, bbf::archive::ArchiveFormat::Cb7).unwrap();
        builder.add_page(&img_a, "a.png").unwrap();
        builder.add_page(&img_b, "b.png").unwrap();
        builder.finalize().unwrap();
    }
    assert!(cb7_path.exists());

    let cb7_file = File::open(&cb7_path).unwrap();
    let cb7_len = cb7_file.metadata().unwrap().len();
    let mut seven_z = sevenz_rust::SevenZReader::new(cb7_file, cb7_len, sevenz_rust::Password::empty()).unwrap();
    let mut file_names = Vec::new();
    seven_z.for_each_entries(|entry, _reader| {
        file_names.push(entry.name().to_string());
        Ok(true)
    }).unwrap();
    assert_eq!(file_names, vec!["a.png", "b.png"]);
}

