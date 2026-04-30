import 'dart:collection';
export 'dart:math';

typedef DartName = String;

const DartLimit = 4;
var dartState = 'ready';

mixin DartRenderable {
  String render(String prefix);
}

enum DartMode {
  compact,
}

class DartWidget with DartRenderable {
  final DartName name;

  DartWidget(
    this.name,
  );

  @override
  String render(
    String prefix,
  ) {
    final ignored = 'class DartStringFake';
    return '$prefix$name';
  }
}

DartWidget buildDartWidget(
  DartName name,
) {
  return DartWidget(name);
}
