import { JavaScriptWidget } from "./widget.js";

function JsxPanel(
  props
) {
  return <section className="jsx-panel">{props.title}</section>;
}

const useJsxPanel = () => {
  return JavaScriptWidget;
};

export { JsxPanel, useJsxPanel };
