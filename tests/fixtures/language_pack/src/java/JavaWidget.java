package fixtures.java;

import java.util.List;

class JavaWidget {
    private static final int JAVA_LIMIT = 4;

    JavaWidget() {
    }

    void render() {
        String ignored = "class JavaStringFake {}";
    }
}

interface JavaRenderable {
    void render();
}

enum JavaMode {
    COMPACT
}

record JavaRecord(
    String name
) {
}
