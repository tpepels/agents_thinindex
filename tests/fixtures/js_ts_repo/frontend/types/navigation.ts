import type { RemoteNavigationProps as RemoteProps } from "./remote";

export interface NavigationProps {
  title: string;
  remote?: RemoteProps;
}

export type NavigationMode = "compact" | "full";

export const createNavigationConfig = function () {
  return { mode: "compact" as NavigationMode };
};
