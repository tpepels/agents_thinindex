import type { TypeScriptDependency } from "./dependency";

export interface TypeScriptRenderable {
  render(): string;
}

type TypeScriptMode = "compact" | "full";

const TS_LIMIT = 4;

class TypeScriptWidget implements TypeScriptRenderable {
  render() {
    return "function TypeScriptStringFake() {}";
  }
}

function buildTypeScriptWidget(
  mode: TypeScriptMode,
): TypeScriptWidget {
  return new TypeScriptWidget();
}

const TypeScriptFactory = () => {
  return TypeScriptWidget;
};

export { TypeScriptWidget, buildTypeScriptWidget, TypeScriptFactory };
