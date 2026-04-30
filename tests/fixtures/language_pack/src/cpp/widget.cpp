#include "widget.hpp"

namespace languagepack {

const int CPP_LIMIT = 4;

class CppWidget {
public:
    void render() {
        const char *ignored = "class CppStringFake {}";
    }
};

struct CppOptions {
    int limit;
};

enum class CppMode {
    Compact
};

int build_cpp_widget(
    int value
) {
    return value;
}

}
