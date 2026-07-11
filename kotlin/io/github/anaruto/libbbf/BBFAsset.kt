package io.github.anaruto.libbbf

class BBFAsset(
    val fileOffset: Long,
    val hash: ByteArray,
    val fileSize: Long,
    val flags: Int,
    val type: Int
)
