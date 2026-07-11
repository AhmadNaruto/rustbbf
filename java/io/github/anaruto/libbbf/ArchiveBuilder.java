package io.github.anaruto.libbbf;

public class ArchiveBuilder implements AutoCloseable {
    static {
        System.loadLibrary("bbf");
    }

    private long handle;

    public ArchiveBuilder(String outputPath, ArchiveFormat format) {
        this.handle = createNative(outputPath, format.ordinal());
        if (this.handle == 0) {
            throw new RuntimeException("Failed to create ArchiveBuilder for " + outputPath);
        }
    }

    @Override
    public void close() {
        if (this.handle != 0) {
            closeNative(this.handle);
            this.handle = 0;
        }
    }

    public boolean addPage(String filePath, String nameInArchive) {
        if (this.handle == 0) {
            throw new IllegalStateException("Builder is closed or finalized");
        }
        return addPage(this.handle, filePath, nameInArchive);
    }

    public void finalizeBuilder() {
        if (this.handle == 0) {
            throw new IllegalStateException("Builder is closed or finalized");
        }
        boolean success = finalizeNative(this.handle);
        this.handle = 0; // Consumed by finalizeNative
        if (!success) {
            throw new RuntimeException("Failed to finalize archive container");
        }
    }

    private static native long createNative(String outputPath, int format);
    private static native void closeNative(long handle);
    private static native boolean addPage(long handle, String filePath, String nameInArchive);
    private static native boolean finalizeNative(long handle);
}
