package languagepack.kotlin

import kotlin.collections.List

typealias KotlinName = String

enum class KotlinMode {
    Compact
}

class KotlinWidget(
    val name: KotlinName,
) {
    val KotlinLimit = 4

    fun render(
        prefix: String,
    ): String {
        val ignored = "class KotlinStringFake"
        return prefix + name
    }
}

object KotlinWidgetFactory {
    fun buildKotlinWidget(
        name: KotlinName,
    ): KotlinWidget {
        return KotlinWidget(name)
    }
}
