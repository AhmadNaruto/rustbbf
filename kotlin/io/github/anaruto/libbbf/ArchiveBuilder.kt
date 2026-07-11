package io.github.anaruto.libbbf

class ArchiveBuilder(outputPath: String, format: ArchiveFormat) : AutoCloseable {
    private var handle: Long = 0L

    init {
        handle = createNative(outputPath, format.ordinal)
        if (handle == 0L) {
            throw RuntimeException("Failed to create ArchiveBuilder for $outputPath")
        }
    }

    override fun close() {
        if (handle != 0L) {
            closeNative(handle)
            handle = 0L
        }
    }

    fun addPage(filePath: String, nameInArchive: String): Boolean {
        check(handle != 0L) { "Builder is closed or finalized" }
        return addPage(handle, filePath, nameInArchive)
    }

    fun finalizeBuilder() {
        check(handle != 0L) { "Builder is closed or finalized" }
        val success = finalizeNative(handle)
        handle = 0L // consumed by finalizeNative
        if (!success) {
            throw RuntimeException("Failed to finalize archive container")
        }
    }

    companion object {
        init {
            System.loadLibrary("bbf")
        }

        @JvmStatic
        private external fun createNative(outputPath: String, format: Int): Long

        @JvmStatic
        private external fun closeNative(handle: Long)

        @JvmStatic
        private external fun addPage(handle: Long, filePath: String, nameInArchive: String): Boolean

        @JvmStatic
        private external fun finalizeNative(handle: Long): Boolean
    }
}
