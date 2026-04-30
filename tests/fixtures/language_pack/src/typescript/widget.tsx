import { TypeScriptWidget } from "./widget";

function TsxPanel(
  props: { title: string },
) {
  return <section className="tsx-panel">{props.title}</section>;
}

const useTsxPanel = () => {
  return TypeScriptWidget;
};

export { TsxPanel, useTsxPanel };
