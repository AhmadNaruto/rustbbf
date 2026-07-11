# libbbf: Bound Book Format (Rust JNI Native Port)

Bound Book Format (`.bbf`) is a binary container format designed for the ordered, deduplicated, and fast indexed storage of page-based media assets such as comics, manga, artbooks, and sequential image collections.

This project is a **pure Rust port** of `libbbf`, designed to compile as a native JNI dynamic library (`.so`) for mobile Android applications.

> [!NOTE]
> The original C++ reference implementation of `libbbf` is available at the original repository: [ef1500/libbbf](https://github.com/ef1500/libbbf).

---

## Features
- **Pure Rust Implementation**: Safer, faster parsing without raw pointer casting risks.
- **Android JNI Native Bindings**: Direct integration with Java/Kotlin mobile apps via standard JNI declarations.
- **Memory-Mapped Reading**: Uses memory-mapped files (`memmap2`) for extreme page-retrieval performance.
- **Zero-Copy Direct Buffer Access**: Exposes native direct memory accesses to JVM (`ByteBuffer`) to bypass garbage-collector memory copies.
- **Asset Deduplication**: Automatically hashes file payloads using XXH3-128 to ensure duplicate pages only occupy disk space once.
- **Integrity Checks**: Validates index table consistency using XXH3-64 footer checksums.
- **Linearization (Petrification)**: Relocates index structures to the beginning of the file for fast streaming and single-read page retrieval.

---

## Project Structure
- `src/codec.rs`: Logika binary parsing, Reader, Builder, dan Petrification dalam Rust.
- `src/jni_api.rs`: Native JNI entrypoints (`Java_io.github.anaruto.libbbf_...`).
- `java/io.github.anaruto.libbbf/`: Kelas wrapper Java (`BBFReader.java`, `BBFBuilder.java`, dll.) untuk diintegrasikan ke proyek Android Studio Anda.
- `tests/codec_tests.rs`: Integration tests untuk memvalidasi alur roundtrip container BBF.

---

## Build & Integration

### 1. Integration Test (Local)
Untuk menjalankan tes integrasi dan memastikan fungsionalitas core codec Rust bekerja sempurna:
```bash
cargo test
```

### 2. Cross-Compile untuk Android
Gunakan `cargo-ndk` untuk mengompilasi dynamic library (`.so`) untuk arsitektur target perangkat Android:

```bash
# 1. Install toolchain target
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# 2. Install helper cargo-ndk
cargo install cargo-ndk

# 3. Lakukan build library native
cargo ndk -t aarch64-linux-android -t armv7-linux-androideabi -t x86_64-linux-android -t i686-linux-android build --release
```

Library `.so` hasil kompilasi akan diletakkan di `target/<architecture>/release/libbbf.so`. Salin file ini ke folder `app/src/main/jniLibs/<architecture>/` di proyek Android Anda.

---

## JNI Usage Example (Java / Kotlin / Android)

Salin kelas wrapper di [java/io.github.anaruto.libbbf/](java/io.github.anaruto.libbbf/) (untuk Java) atau [kotlin/io.github.anaruto.libbbf/](kotlin/io.github.anaruto.libbbf/) (untuk Kotlin) ke dalam proyek Android Anda.

### Java Example

#### Membaca File BBF
```java
import io.github.anaruto.libbbf.BBFReader;
import io.github.anaruto.libbbf.BBFAsset;
import java.nio.ByteBuffer;

try (BBFReader reader = new BBFReader("/sdcard/Documents/manga.bbf")) {
    if (!reader.verifyFooterHash()) {
        throw new RuntimeException("File BBF korup!");
    }

    long pageCount = reader.getPageCount();
    for (int i = 0; i < pageCount; i++) {
        long assetIndex = reader.getPageAssetIndex(i);
        BBFAsset asset = reader.getAsset((int) assetIndex);
        
        byte[] imageBytes = reader.getAssetData((int) assetIndex);
        
        ByteBuffer directBuffer = ByteBuffer.allocateDirect((int) asset.fileSize);
        reader.readAssetDataDirect((int) assetIndex, directBuffer, 0, (int) asset.fileSize);
    }
}
```

#### Membuat File BBF
```java
import io.github.anaruto.libbbf.BBFBuilder;

try (BBFBuilder builder = new BBFBuilder("/sdcard/Documents/output.bbf", 12, 16, 2)) {
    builder.addPage("/sdcard/Pictures/page1.png", 0, 0);
    builder.addPage("/sdcard/Pictures/page2.png", 0, 0);
    
    builder.addMeta("Title", "My Manga Vol. 1", null);
    builder.addMeta("Author", "Illustrator", null);
    
    builder.addSection("Chapter 1", 0, null);
    builder.finalizeBuilder();
}
```

### Kotlin Example

#### Membaca File BBF (Zero-copy direct ByteBuffer)
```kotlin
import io.github.anaruto.libbbf.BBFReader
import java.nio.ByteBuffer

BBFReader("/sdcard/Documents/manga.bbf").use { reader ->
    if (!reader.verifyFooterHash()) {
        throw RuntimeException("File BBF korup!")
    }

    val pageCount = reader.pageCount
    for (i in 0 until pageCount) {
        val assetIndex = reader.getPageAssetIndex(i.toInt())
        val asset = reader.getAsset(assetIndex.toInt())
        
        // Membaca data mentah (ByteArray)
        val imageBytes = reader.getAssetData(assetIndex.toInt())
        
        // ATAU membaca langsung ke direct ByteBuffer (zero-copy)
        val directBuffer = ByteBuffer.allocateDirect(asset.fileSize.toInt())
        reader.readAssetDataDirect(assetIndex.toInt(), directBuffer, 0, asset.fileSize.toInt())
    }
}
```

#### Membuat File BBF
```kotlin
import io.github.anaruto.libbbf.BBFBuilder

BBFBuilder("/sdcard/Documents/output.bbf", 12, 16, 2).use { builder ->
    builder.addPage("/sdcard/Pictures/page1.png", 0, 0)
    builder.addPage("/sdcard/Pictures/page2.png", 0, 0)
    
    builder.addMeta("Title", "My Manga Vol. 1", null)
    builder.addMeta("Author", "Illustrator", null)
    
    builder.addSection("Chapter 1", 0, null)
    builder.finalizeBuilder()
}
```

---

## Technical Specifications
Format file mengikuti spesifikasi **BBF v3.0.1** (Little-Endian).

### Layout Default (Non-Petrified)
```text
[BBF Header] (64 bytes)
[Asset Data (Aligned)]
[Asset Table]
[Page Table]
[Section Table]
[Metadata Table]
[Expansion Table] (Optional)
[String Pool]
[BBF Footer] (256 bytes)
```

### Layout Linearized (Petrified)
Untuk pemuatan instan atau streaming jaringan (membaca header & footer hanya dalam satu blok baca 320 byte pertama):
```text
[BBF Header]
[BBF Footer]
[Asset Table]
[Page Table]
[Section Table]
[Metadata Table]
[Expansion Table] (Optional)
[String Table]
[Asset Data (Aligned)]
```

Untuk mengubah file kontainer standar ke layout petrified:
```java
boolean success = BBFBuilder.petrify("/sdcard/Documents/manga.bbf", "/sdcard/Documents/manga_petrified.bbf");
```

---

## License
Distributed under the MIT License. See `LICENSE` for more information.
