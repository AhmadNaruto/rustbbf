package io.github.anaruto.libbbf;

public class BBFAsset {
    public final long fileOffset;
    public final byte[] hash;
    public final long fileSize;
    public final int flags;
    public final int type;

    public BBFAsset(long fileOffset, byte[] hash, long fileSize, int flags, int type) {
        this.fileOffset = fileOffset;
        this.hash = hash;
        this.fileSize = fileSize;
        this.flags = flags;
        this.type = type;
    }
}
