package io.github.anaruto.libbbf;

import java.nio.ByteBuffer;

public class BBFReader implements AutoCloseable {
    static {
        System.loadLibrary("bbf");
    }

    private final long handle;

    public BBFReader(String filePath) {
        this.handle = openNative(filePath);
        if (this.handle == 0) {
            throw new RuntimeException("Failed to open BBF container at " + filePath);
        }
    }

    public BBFReader(byte[] bytes) {
        this.handle = openBytesNative(bytes);
        if (this.handle == 0) {
            throw new RuntimeException("Failed to open BBF container from bytes");
        }
    }

    @Override
    public void close() {
        closeNative(handle);
    }

    public int getVersion() {
        return getVersion(handle);
    }

    public int getHeaderFlags() {
        return getHeaderFlags(handle);
    }

    public int getAlignment() {
        return getAlignment(handle);
    }

    public int getReamSize() {
        return getReamSize(handle);
    }

    public long getAssetCount() {
        return getAssetCount(handle);
    }

    public long getPageCount() {
        return getPageCount(handle);
    }

    public long getSectionCount() {
        return getSectionCount(handle);
    }

    public long getMetaCount() {
        return getMetaCount(handle);
    }

    public BBFMeta getMeta(int index) {
        String key = getMetaKey(handle, index);
        String val = getMetaValue(handle, index);
        String parent = getMetaParent(handle, index);
        return new BBFMeta(key, val, parent);
    }

    public BBFSection getSection(int index) {
        String title = getSectionTitle(handle, index);
        long start = getSectionStartIndex(handle, index);
        String parent = getSectionParent(handle, index);
        return new BBFSection(title, start, parent);
    }

    public long getPageAssetIndex(int index) {
        return getPageAssetIndex(handle, index);
    }

    public int getPageFlags(int index) {
        return getPageFlags(handle, index);
    }

    public BBFAsset getAsset(int index) {
        long offset = getAssetFileOffset(handle, index);
        long size = getAssetFileSize(handle, index);
        int flags = getAssetFlags(handle, index);
        int type = getAssetType(handle, index);
        byte[] hash = getAssetHash(handle, index);
        return new BBFAsset(offset, hash, size, flags, type);
    }

    public byte[] getAssetData(int index) {
        return getAssetData(handle, index);
    }

    public int readAssetDataDirect(int index, ByteBuffer buffer, int offset, int len) {
        if (!buffer.isDirect()) {
            throw new IllegalArgumentException("ByteBuffer must be a direct buffer");
        }
        return readAssetDataDirect(handle, index, buffer, offset, len);
    }

    public boolean verifyFooterHash() {
        return verifyFooterHash(handle);
    }

    public boolean verifyAssetHash(int index) {
        return verifyAssetHash(handle, index);
    }

    // JNI Declarations
    private static native long openNative(String filePath);
    private static native long openBytesNative(byte[] bytes);
    private static native void closeNative(long handle);

    private static native int getVersion(long handle);
    private static native int getHeaderFlags(long handle);
    private static native int getAlignment(long handle);
    private static native int getReamSize(long handle);

    private static native long getAssetCount(long handle);
    private static native long getPageCount(long handle);
    private static native long getSectionCount(long handle);
    private static native long getMetaCount(long handle);

    private static native String getMetaKey(long handle, int index);
    private static native String getMetaValue(long handle, int index);
    private static native String getMetaParent(long handle, int index);

    private static native String getSectionTitle(long handle, int index);
    private static native long getSectionStartIndex(long handle, int index);
    private static native String getSectionParent(long handle, int index);

    private static native long getPageAssetIndex(long handle, int index);
    private static native int getPageFlags(long handle, int index);

    private static native long getAssetFileOffset(long handle, int index);
    private static native long getAssetFileSize(long handle, int index);
    private static native int getAssetFlags(long handle, int index);
    private static native int getAssetType(long handle, int index);
    private static native byte[] getAssetHash(long handle, int index);

    private static native byte[] getAssetData(long handle, int index);
    private static native int readAssetDataDirect(long handle, int index, ByteBuffer buffer, int offset, int len);

    private static native boolean verifyFooterHash(long handle);
    private static native boolean verifyAssetHash(long handle, int index);
}
