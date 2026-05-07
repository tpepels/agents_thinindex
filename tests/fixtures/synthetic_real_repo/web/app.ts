import { buildSyntheticWidget } from "./widget";

export * from "./widget";

export async function renderApp(): Promise<string> {
  const lazy = await import("./lazy");
  return `${buildSyntheticWidget().render()}-${lazy.lazyValue}`;
}
