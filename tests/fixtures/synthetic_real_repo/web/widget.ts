export interface WidgetView {
  render(): string;
}

export class SyntheticWidget implements WidgetView {
  render(): string {
    return "synthetic";
  }
}

export function buildSyntheticWidget(): SyntheticWidget {
  return new SyntheticWidget();
}
