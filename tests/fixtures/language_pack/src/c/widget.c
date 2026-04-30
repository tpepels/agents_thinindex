#include "widget.h"

const int C_LIMIT = 4;

typedef struct CWidget {
    int value;
} CWidget;

enum CMode {
    C_MODE_COMPACT
};

int build_c_widget(
    int value
) {
    const char *ignored = "int CStringFake(void) {}";
    return value;
}
