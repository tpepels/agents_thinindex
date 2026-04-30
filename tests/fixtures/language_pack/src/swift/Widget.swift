import Foundation

typealias SwiftName = String

protocol SwiftRenderable {
    func render(
        prefix: String
    ) -> String
}

enum SwiftMode {
    case compact
}

struct SwiftPayload {
    let value: String
}

class SwiftWidget: SwiftRenderable {
    let SwiftLimit = 4
    let name: SwiftName

    init(
        name: SwiftName
    ) {
        self.name = name
    }

    func render(
        prefix: String
    ) -> String {
        let ignored = "class SwiftStringFake"
        return prefix + name
    }
}

func buildSwiftWidget(
    name: SwiftName
) -> SwiftWidget {
    return SwiftWidget(name: name)
}
