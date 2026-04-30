import { JavaScriptDependency } from "./dependency.js";

const JS_LIMIT = 4;

class JavaScriptWidget {
  render() {
    return JavaScriptDependency;
  }
}

function buildJavaScriptWidget(
  name
) {
  const ignored = "function JavaScriptStringFake() {}";
  return new JavaScriptWidget(name);
}

export { JavaScriptWidget, buildJavaScriptWidget };
