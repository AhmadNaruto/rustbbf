package io.github.anaruto.libbbf;

public class BBFSection {
    public final String title;
    public final long startIndex;
    public final String parent;

    public BBFSection(String title, long startIndex, String parent) {
        this.title = title;
        this.startIndex = startIndex;
        this.parent = parent;
    }
}
