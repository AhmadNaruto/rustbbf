package io.github.anaruto.libbbf;

public class BBFBuilder implements AutoCloseable {
    static {
        System.loadLibrary("bbf");
    }

    private long handle;

    public BBFBuilder(String outputPath, int alignment, int reamSize, int flags) {
        this.handle = createNative(outputPath, alignment, reamSize, flags);
        if (this.handle == 0) {
            throw new RuntimeException("Failed to create BBFBuilder for " + outputPath);
        }
    }

    @Override
    public void close() {
        if (handle != 0) {
            closeNative(handle);
            handle = 0;
        }
    }

    public boolean addPage(String filePath, int pageFlags, int assetFlags) {
        if (handle == 0) throw new IllegalStateException("Builder is closed or finalized");
        return addPage(handle, filePath, pageFlags, assetFlags);
    }

    public boolean addMeta(String key, String value, String parent) {
        if (handle == 0) throw new IllegalStateException("Builder is closed or finalized");
        return addMeta(handle, key, value, parent);
    }

    public boolean addSection(String name, long startIndex, String parent) {
        if (handle == 0) throw new IllegalStateException("Builder is closed or finalized");
        return addSection(handle, name, startIndex, parent);
    }

    public void finalizeBuilder() {
        if (handle == 0) throw new IllegalStateException("Builder is closed or finalized");
        boolean success = finalizeNative(handle);
        handle = 0; // consumed by finalizeNative
        if (!success) {
            throw new RuntimeException("Failed to finalize BBF container");
        }
    }

    public static boolean petrify(String inputPath, String outputPath) {
        return petrifyNative(inputPath, outputPath);
    }

    // JNI Declarations
    private static native long createNative(String outputPath, int alignment, int reamSize, int flags);
    private static native void closeNative(long handle);
    private static native boolean addPage(long handle, String filePath, int pageFlags, int assetFlags);
    private static native boolean addMeta(long handle, String key, String value, String parent);
    private static native boolean addSection(long handle, String name, long startIndex, String parent);
    private static native boolean finalizeNative(long handle);
    private static native boolean petrifyNative(String inputPath, String outputPath);
}
