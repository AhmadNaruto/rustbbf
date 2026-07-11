package io.github.anaruto.libbbf

import java.nio.ByteBuffer

class BBFReader : AutoCloseable {
    private val handle: Long

    constructor(filePath: String) {
        handle = openNative(filePath)
        if (handle == 0L) {
            throw RuntimeException("Failed to open BBF container at $filePath")
        }
    }

    constructor(bytes: ByteArray) {
        handle = openBytesNative(bytes)
        if (handle == 0L) {
            throw RuntimeException("Failed to open BBF container from bytes")
        }
    }

    override fun close() {
        if (handle != 0L) {
            closeNative(handle)
        }
    }

    val version: Int
        get() = getVersion(handle)

    val headerFlags: Int
        get() = getHeaderFlags(handle)

    val alignment: Int
        get() = getAlignment(handle)

    val reamSize: Int
        get() = getReamSize(handle)

    val assetCount: Long
        get() = getAssetCount(handle)

    val pageCount: Long
        get() = getPageCount(handle)

    val sectionCount: Long
        get() = getSectionCount(handle)

    val metaCount: Long
        get() = getMetaCount(handle)

    fun getMeta(index: Int): BBFMeta {
        val key = getMetaKey(handle, index) ?: ""
        val value = getMetaValue(handle, index) ?: ""
        val parent = getMetaParent(handle, index).takeIf { !it.isNullOrEmpty() }
        return BBFMeta(key, value, parent)
    }

    fun getSection(index: Int): BBFSection {
        val title = getSectionTitle(handle, index) ?: ""
        val startIndex = getSectionStartIndex(handle, index)
        val parent = getSectionParent(handle, index).takeIf { !it.isNullOrEmpty() }
        return BBFSection(title, startIndex, parent)
    }

    fun getPageAssetIndex(index: Int): Long {
        return getPageAssetIndex(handle, index)
    }

    fun getPageFlags(index: Int): Int {
        return getPageFlags(handle, index)
    }

    fun getAsset(index: Int): BBFAsset {
        val offset = getAssetFileOffset(handle, index)
        val size = getAssetFileSize(handle, index)
        val flags = getAssetFlags(handle, index)
        val type = getAssetType(handle, index)
        val hash = getAssetHash(handle, index) ?: ByteArray(16)
        return BBFAsset(offset, hash, size, flags, type)
    }

    fun getAssetData(index: Int): ByteArray? {
        return getAssetData(handle, index)
    }

    fun readAssetDataDirect(index: Int, buffer: ByteBuffer, offset: Int, len: Int): Int {
        if (!buffer.isDirect) {
            throw IllegalArgumentException("ByteBuffer must be a direct buffer")
        }
        return readAssetDataDirect(handle, index, buffer, offset, len)
    }

    fun verifyFooterHash(): Boolean {
        return verifyFooterHash(handle)
    }

    fun verifyAssetHash(index: Int): Boolean {
        return verifyAssetHash(handle, index)
    }

    companion object {
        init {
            System.loadLibrary("bbf")
        }

        @JvmStatic
        private external fun openNative(filePath: String): Long

        @JvmStatic
        private external fun openBytesNative(bytes: ByteArray): Long

        @JvmStatic
        private external fun closeNative(handle: Long)

        @JvmStatic
        private external fun getVersion(handle: Long): Int

        @JvmStatic
        private external fun getHeaderFlags(handle: Long): Int

        @JvmStatic
        private external fun getAlignment(handle: Long): Int

        @JvmStatic
        private external fun getReamSize(handle: Long): Int

        @JvmStatic
        private external fun getAssetCount(handle: Long): Long

        @JvmStatic
        private external fun getPageCount(handle: Long): Long

        @JvmStatic
        private external fun getSectionCount(handle: Long): Long

        @JvmStatic
        private external fun getMetaCount(handle: Long): Long

        @JvmStatic
        private external fun getMetaKey(handle: Long, index: Int): String?

        @JvmStatic
        private external fun getMetaValue(handle: Long, index: Int): String?

        @JvmStatic
        private external fun getMetaParent(handle: Long, index: Int): String?

        @JvmStatic
        private external fun getSectionTitle(handle: Long, index: Int): String?

        @JvmStatic
        private external fun getSectionStartIndex(handle: Long, index: Int): Long

        @JvmStatic
        private external fun getSectionParent(handle: Long, index: Int): String?

        @JvmStatic
        private external fun getPageAssetIndex(handle: Long, index: Int): Long

        @JvmStatic
        private external fun getPageFlags(handle: Long, index: Int): Int

        @JvmStatic
        private external fun getAssetFileOffset(handle: Long, index: Int): Long

        @JvmStatic
        private external fun getAssetFileSize(handle: Long, index: Int): Long

        @JvmStatic
        private external fun getAssetFlags(handle: Long, index: Int): Int

        @JvmStatic
        private external fun getAssetType(handle: Long, index: Int): Int

        @JvmStatic
        private external fun getAssetHash(handle: Long, index: Int): ByteArray?

        @JvmStatic
        private external fun getAssetData(handle: Long, index: Int): ByteArray?

        @JvmStatic
        private external fun readAssetDataDirect(handle: Long, index: Int, buffer: ByteBuffer, offset: Int, len: Int): Int

        @JvmStatic
        private external fun verifyFooterHash(handle: Long): Boolean

        @JvmStatic
        private external fun verifyAssetHash(handle: Long, index: Int): Boolean
    }
}
