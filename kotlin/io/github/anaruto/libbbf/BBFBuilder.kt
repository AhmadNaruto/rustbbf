package io.github.anaruto.libbbf

class BBFBuilder(outputPath: String, alignment: Int, reamSize: Int, flags: Int) : AutoCloseable {
    private var handle: Long = 0L

    init {
        handle = createNative(outputPath, alignment, reamSize, flags)
        if (handle == 0L) {
            throw RuntimeException("Failed to create BBFBuilder for $outputPath")
        }
    }

    override fun close() {
        if (handle != 0L) {
            closeNative(handle)
            handle = 0L
        }
    }

    fun addPage(filePath: String, pageFlags: Int, assetFlags: Int): Boolean {
        check(handle != 0L) { "Builder is closed or finalized" }
        return addPage(handle, filePath, pageFlags, assetFlags)
    }

    fun addMeta(key: String, value: String, parent: String?): Boolean {
        check(handle != 0L) { "Builder is closed or finalized" }
        return addMeta(handle, key, value, parent)
    }

    fun addSection(name: String, startIndex: Long, parent: String?): Boolean {
        check(handle != 0L) { "Builder is closed or finalized" }
        return addSection(handle, name, startIndex, parent)
    }

    fun finalizeBuilder() {
        check(handle != 0L) { "Builder is closed or finalized" }
        val success = finalizeNative(handle)
        handle = 0L // consumed by finalizeNative
        if (!success) {
            throw RuntimeException("Failed to finalize BBF container")
        }
    }

    companion object {
        init {
            System.loadLibrary("bbf")
        }

        @JvmStatic
        fun petrify(inputPath: String, outputPath: String): Boolean {
            return petrifyNative(inputPath, outputPath)
        }

        @JvmStatic
        private external fun createNative(outputPath: String, alignment: Int, reamSize: Int, flags: Int): Long

        @JvmStatic
        private external fun closeNative(handle: Long)

        @JvmStatic
        private external fun addPage(handle: Long, filePath: String, pageFlags: Int, assetFlags: Int): Boolean

        @JvmStatic
        private external fun addMeta(handle: Long, key: String, value: String, parent: String?): Boolean

        @JvmStatic
        private external fun addSection(handle: Long, name: String, startIndex: Long, parent: String?): Boolean

        @JvmStatic
        private external fun finalizeNative(handle: Long): Boolean

        @JvmStatic
        private external fun petrifyNative(inputPath: String, outputPath: String): Boolean
    }
}
